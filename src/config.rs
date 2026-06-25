use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// These are for overriding settings from the config.toml
#[derive(Debug, Parser)]
#[command(name = "irma-viz", version, about = "Render IRMA plots to SVG")]
pub struct Args {
    #[command(flatten)]
    pub io_args: IOArgs,
    /// Path to config TOML
    #[arg(long, default_value = "config.toml")]
    pub config: String,
    /// Which figures to plot
    #[command(flatten)]
    pub enabled_plots: PlotToggleArgs,
    /// Constants for heuristic plot
    #[command(flatten)]
    pub constants_args: ConstantsArgs,
    /// Plot specific args
    #[command(flatten)]
    pub plot_specific_args: PlotSpecificArgs,
}

#[derive(Debug, Parser, Clone)]
pub struct IOArgs {
    /// Path to input directory that contains `tables/` and `matrices/`
    #[arg(long, short = 'i')]
    pub input_root: PathBuf,
    /// Destination directory for output figures. If not specified, defaults to
    /// `input_root/figures/`
    #[arg(long, short = 'o')]
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    // this is skipped for deserialization because we don't actually expect it
    // in the toml, but we do want to keep it in the Config object
    #[serde(skip)]
    pub io_args: Option<IOArgs>,
    pub plot_toggles: PlotToggles,
    pub constants: ConstantsConfig,

    #[serde(flatten)]
    pub plot_specific: PlotSpecificConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlotToggles {
    pub read_percentages: bool,
    pub heuristics: bool,
    pub coverage: bool,
    pub clustermap: bool,
}

// toggles for enabling/disabling to override the config
// if these flags aren't used, the default will stick
// e.g.
//   `--coverage true`
#[derive(Debug, Parser)]
pub struct PlotToggleArgs {
    #[arg(long)]
    pub read_percentages: Option<bool>,
    #[arg(long)]
    pub heuristics: Option<bool>,
    #[arg(long)]
    pub coverage: Option<bool>,
    #[arg(long)]
    pub clustermap: Option<bool>,
}

#[derive(Debug, Parser, Deserialize)]
pub struct ConstantsArgs {
    #[arg(long)]
    pub min_aq: Option<f64>,
    #[arg(long)]
    pub min_f: Option<f64>,
    #[arg(long)]
    pub min_tcc: Option<f64>,
    #[arg(long)]
    pub min_conf: Option<f64>,
    #[arg(long)]
    pub tree_height: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ConstantsConfig {
    pub min_aq: f64,
    pub min_f: f64,
    pub min_tcc: f64,
    pub min_conf: f64,
    pub tree_height: f64,
}

#[derive(Debug, Deserialize)]
pub struct PlotSpecificConfig {
    #[serde(rename = "coverage_options")]
    pub coverage: CoverageConfig,

    #[serde(rename = "percent_options")]
    pub read_percent: ReadPercentConfig,

    #[serde(rename = "cluster_options")]
    pub cluster_config: ClusterConfig,
}

#[derive(Debug, Parser)]
pub struct PlotSpecificArgs {
    #[arg(long, value_enum)]
    pub coverage_variant_color: Option<CoverageColorOption>,

    #[arg(long, value_enum)]
    pub read_percentages_viz: Option<PercentVizOption>,

    #[arg(long)]
    pub paired: Option<bool>,

    #[arg(long)]
    pub cluster_option: Option<ClusterOption>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CoverageColorOption {
    Nucleotide,
    Frequency,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum PercentVizOption {
    Sankey,
    Pie,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ClusterOption {
    Clustermap,
    Tree,
}

#[derive(Debug, Deserialize)]
pub struct CoverageConfig {
    #[serde(rename = "variant_color")]
    pub color_option: CoverageColorOption,
}

#[derive(Debug, Deserialize)]
pub struct ClusterConfig {
    pub cluster_option: ClusterOption,
    pub matrix_types: MatrixTypes,
}

#[derive(Debug, Deserialize)]
pub struct MatrixTypes {
    pub expenrd: bool,
    pub jaccard: bool,
    pub mutuald: bool,
    pub njointp: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MatrixType {
    Expenrd,
    Jaccard,
    Mutuald,
    Njointp,
}

/// Converts the struct of bools created from the config.toml into a Vec of
/// enabled matrix types for iteration
impl MatrixTypes {
    pub fn enabled_matrix_types(&self) -> Vec<MatrixType> {
        let mut enabled = Vec::new();
        if self.expenrd {
            enabled.push(MatrixType::Expenrd);
        }
        if self.jaccard {
            enabled.push(MatrixType::Jaccard);
        }
        if self.mutuald {
            enabled.push(MatrixType::Mutuald);
        }
        if self.njointp {
            enabled.push(MatrixType::Njointp);
        }
        enabled
    }
}

impl MatrixType {
    pub fn display_name(self) -> &'static str {
        match self {
            MatrixType::Expenrd => "EXPENRD",
            MatrixType::Jaccard => "JACCARD",
            MatrixType::Mutuald => "MUTUALD",
            MatrixType::Njointp => "NJOINTP",
        }
    }

    pub fn file_suffix(self) -> &'static str {
        match self {
            MatrixType::Expenrd => "-EXPENRD.sqm",
            MatrixType::Jaccard => "-JACCARD.sqm",
            MatrixType::Mutuald => "-MUTUALD.sqm",
            MatrixType::Njointp => "-NJOINTP.sqm",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReadPercentConfig {
    pub viz_option: PercentVizOption,
    pub paired: bool,
}

fn merge<T>(target: &mut T, override_val: Option<T>) {
    if let Some(v) = override_val {
        *target = v;
    }
}

pub fn apply_cli_overrides(mut cfg: Config, args: &Args) -> Config {
    cfg.io_args = Some(args.io_args.clone());

    // plot overrides
    merge(
        &mut cfg.plot_toggles.read_percentages,
        args.enabled_plots.read_percentages,
    );
    merge(
        &mut cfg.plot_toggles.heuristics,
        args.enabled_plots.heuristics,
    );
    merge(&mut cfg.plot_toggles.coverage, args.enabled_plots.coverage);
    merge(
        &mut cfg.plot_toggles.clustermap,
        args.enabled_plots.clustermap,
    );

    // heuristics constants
    merge(&mut cfg.constants.min_aq, args.constants_args.min_aq);
    merge(&mut cfg.constants.min_f, args.constants_args.min_f);
    merge(&mut cfg.constants.min_conf, args.constants_args.min_conf);
    merge(&mut cfg.constants.min_tcc, args.constants_args.min_tcc);
    merge(
        &mut cfg.constants.tree_height,
        args.constants_args.tree_height,
    );

    // plot-specific
    merge(
        &mut cfg.plot_specific.coverage.color_option,
        args.plot_specific_args.coverage_variant_color,
    );
    merge(
        &mut cfg.plot_specific.read_percent.viz_option,
        args.plot_specific_args.read_percentages_viz,
    );
    merge(
        &mut cfg.plot_specific.read_percent.paired,
        args.plot_specific_args.paired,
    );
    merge(
        &mut cfg.plot_specific.cluster_config.cluster_option,
        args.plot_specific_args.cluster_option,
    );

    cfg
}

pub fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("Error reading \'{path}\'"))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("Error parsing \'{path}\'"))?;
    Ok(cfg)
}

impl Config {
    /// Convenience function for getting IO args from [Config]
    pub fn io_args(&self) -> Result<&IOArgs> {
        self.io_args
            .as_ref()
            .context("IO arguments were not attached to the runtime config")
    }

    /// Convenience function for getting output path from [Config]
    pub fn output_path(&self) -> Result<PathBuf> {
        let io_args = self.io_args()?;
        Ok(io_args
            .output_path
            .clone()
            .unwrap_or_else(|| io_args.input_root.join("figures")))
    }
}

const HEURISTICS_REQUIRED_SUFFIXES: &[&str] = &["-allAlleles.txt"];
const COVERAGE_REQUIRED_TABLE_SUFFIXES: &[&str] =
    &["-variants.txt", "-coverage.txt", "-pairingStats.txt"];
const CLUSTERMAP_REQUIRED_TABLE_SUFFIXES: &[&str] = &["-variants.txt"];

/// Stores a list of targets seperately for each matrix type
#[derive(Debug, Default)]
pub struct ClusterTargets {
    pub expenrd: BTreeSet<String>,
    pub jaccard: BTreeSet<String>,
    pub mutuald: BTreeSet<String>,
    pub njointp: BTreeSet<String>,
}

impl ClusterTargets {
    pub fn insert(&mut self, matrix_type: MatrixType, target: String) {
        self.targets_for_mut(matrix_type).insert(target);
    }

    /// gets the set of targets for a given matrix type
    pub fn targets_for(&self, matrix_type: MatrixType) -> &BTreeSet<String> {
        match matrix_type {
            MatrixType::Expenrd => &self.expenrd,
            MatrixType::Jaccard => &self.jaccard,
            MatrixType::Mutuald => &self.mutuald,
            MatrixType::Njointp => &self.njointp,
        }
    }

    /// gets the set of targets for a given matrix type mutably
    fn targets_for_mut(&mut self, matrix_type: MatrixType) -> &mut BTreeSet<String> {
        match matrix_type {
            MatrixType::Expenrd => &mut self.expenrd,
            MatrixType::Jaccard => &mut self.jaccard,
            MatrixType::Mutuald => &mut self.mutuald,
            MatrixType::Njointp => &mut self.njointp,
        }
    }

    pub fn variant_targets(&self) -> BTreeSet<String> {
        self.expenrd
            .union(&self.jaccard)
            .chain(self.mutuald.union(&self.njointp))
            .cloned()
            .collect()
    }

    // a big ugly way to ensure that if a matrix file exists for a certain
    // matrix type, we warn if it doesn't exist for the other enabled matrix type(s)
    pub fn check_missing_matrix_targets(&self, matrix_types: &MatrixTypes) {
        let enabled = matrix_types.enabled_matrix_types();
        if enabled.len() < 2 {
            return;
        }

        let any_enabled_targets: BTreeSet<String> = enabled
            .iter()
            .flat_map(|matrix_type| self.targets_for(*matrix_type).iter().cloned())
            .collect();

        for target in any_enabled_targets {
            let present_in = enabled
                .iter()
                .filter(|matrix_type| self.targets_for(**matrix_type).contains(&target))
                .map(|matrix_type| matrix_type.display_name())
                .collect::<Vec<_>>();

            if present_in.len() == enabled.len() {
                continue;
            }

            let missing_from = enabled
                .iter()
                .filter(|matrix_type| !self.targets_for(**matrix_type).contains(&target))
                .map(|matrix_type| matrix_type.display_name())
                .collect::<Vec<_>>();

            eprintln!(
                "Warning: clustermap matrix files were found for target {} for enabled matrix {} but not in enabled {}",
                target,
                present_in.join(", "),
                missing_from.join(", ")
            );
        }
    }
}

/// A collection of confirmed targets, stored separately for each plot type
#[derive(Debug, Default)]
pub struct PlotTargets {
    pub heuristics: BTreeSet<String>,
    pub coverage: BTreeSet<String>,
    pub clustermap: ClusterTargets,
}

impl PlotTargets {
    pub fn variant_targets(&self) -> BTreeSet<String> {
        self.coverage
            .union(&self.clustermap.variant_targets())
            .cloned()
            .collect()
    }

    /// cross-check the targets for each plot type to see if there are targets
    /// missing from one plot but not another. Also checks if no targets were
    /// found for a given plot type (excluding clustermap)
    pub fn check_missing_targets(&self, cfg: &Config) {
        let toggles = &cfg.plot_toggles;

        if toggles.heuristics && self.heuristics.is_empty() {
            eprintln!("Warning: heuristics plotting was enabled but no valid targets were found");
        }
        if toggles.coverage && self.coverage.is_empty() {
            eprintln!("Warning: coverage plotting was enabled but not valid targets were found");
        }

        if toggles.heuristics && toggles.coverage {
            warn_missing(&self.heuristics, &self.coverage, "heuristics", "coverage");
            warn_missing(&self.coverage, &self.heuristics, "coverage", "heuristics");
        }

        let clustermap_targets = self.clustermap.variant_targets();

        if toggles.heuristics && toggles.clustermap {
            warn_missing(
                &clustermap_targets,
                &self.heuristics,
                "clustermap",
                "heuristics",
            );
        }

        if toggles.coverage && toggles.clustermap {
            warn_missing(
                &clustermap_targets,
                &self.coverage,
                "clustermap",
                "coverage",
            );
        }

        if toggles.clustermap {
            self.clustermap
                .check_missing_matrix_targets(&cfg.plot_specific.cluster_config.matrix_types);
        }
    }
}

/// helper function for warning of missing files for a given target and plot type
fn warn_missing<'a>(
    from: &'a BTreeSet<String>,
    to: &'a BTreeSet<String>,
    from_name: &str,
    to_name: &str,
) {
    for target in from.difference(to) {
        eprintln!(
            "Warning: necessary files found to create {from_name} plot but not {to_name} plot for target {target}"
        );
    }
}

/// takes the input root for an IRMA run and returns paths for the `tables/` and `matrices/` directories
pub fn get_directory_paths(input_root: &Path) -> (PathBuf, PathBuf) {
    (input_root.join("tables"), input_root.join("matrices"))
}

pub fn resolve_targets(cfg: &Config) -> Result<PlotTargets> {
    // no plots needing targets enabled
    if !(cfg.plot_toggles.heuristics || cfg.plot_toggles.coverage || cfg.plot_toggles.clustermap) {
        return Ok(PlotTargets::default());
    }

    let io_args = cfg.io_args()?;
    let (table_path, matrix_path) = get_directory_paths(&io_args.input_root);
    discover_targets_by_plot_type(
        &table_path,
        &matrix_path,
        &cfg.plot_toggles,
        &cfg.plot_specific.cluster_config.matrix_types,
    )
}

fn discover_targets_by_plot_type(
    table_dir: &Path,
    matrix_dir: &Path,
    plot_toggles: &PlotToggles,
    matrix_types: &MatrixTypes,
) -> Result<PlotTargets> {
    let mut plot_targets = PlotTargets::default();
    // collects all possible heuristics targets
    if plot_toggles.heuristics {
        let possible_heuristics_targets =
            discover_candidate_targets(table_dir, HEURISTICS_REQUIRED_SUFFIXES)?;
        for possible_target in possible_heuristics_targets {
            let required_heuristics_files =
                required_target_files(table_dir, &possible_target, HEURISTICS_REQUIRED_SUFFIXES);
            if validate_target_files(&possible_target, required_heuristics_files, "heuristics") {
                plot_targets.heuristics.insert(possible_target);
            }
        }
    }

    // collects all possible coverage targets
    if plot_toggles.coverage {
        // all of the potential targets we see
        let possible_coverage_targets =
            discover_candidate_targets(table_dir, COVERAGE_REQUIRED_TABLE_SUFFIXES)?;
        // for each possible target, we need to check if we have all of the required files for it
        for possible_target in possible_coverage_targets {
            let required_coverage_files = required_target_files(
                table_dir,
                &possible_target,
                COVERAGE_REQUIRED_TABLE_SUFFIXES,
            );
            if validate_target_files(&possible_target, required_coverage_files, "coverage") {
                plot_targets.coverage.insert(possible_target);
            }
        }
    }

    if plot_toggles.clustermap {
        // we only need to check the matrix directory for targets, since empty
        // variants files will be created for each target even if there's no matrix
        for matrix_type in matrix_types.enabled_matrix_types() {
            let possible_clustermap_targets =
                discover_candidate_targets(matrix_dir, &[matrix_type.file_suffix()])?;

            for possible_target in possible_clustermap_targets {
                // build up a list of theoretical paths, both from the matrix
                // directory and table directory, that all need to exist to create
                // the given target's clustermap
                let mut required = required_target_files(
                    table_dir,
                    &possible_target,
                    CLUSTERMAP_REQUIRED_TABLE_SUFFIXES,
                );
                required.push(
                    matrix_dir.join(format!("{possible_target}{}", matrix_type.file_suffix())),
                );

                if validate_target_files(&possible_target, required, "clustermap") {
                    plot_targets.clustermap.insert(matrix_type, possible_target);
                }
            }
        }
    }
    Ok(plot_targets)
}

/// Takes  a path to a directory and a list of suffixes and returns a BTreeSet
/// of possible targets that have files with these suffixes
fn discover_candidate_targets(dir: &Path, suffixes: &[&str]) -> Result<BTreeSet<String>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Error reading input directory '{}'", dir.display()))?;
    let mut targets = BTreeSet::new();

    for entry in entries {
        let entry =
            entry.with_context(|| format!("Error reading entry from '{}'", dir.display()))?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };

        for suffix in suffixes {
            if let Some(target) = file_name.strip_suffix(suffix) {
                if is_valid_target_name(target) {
                    targets.insert(target.to_owned());
                } else {
                    eprintln!(
                        "Warning: skipping target derived from {file_name:?}: invalid target name {target:?}"
                    );
                }
                break;
            }
        }
    }

    Ok(targets)
}

/// Creates a list of theoretical paths for required target files to make a
/// certain plot, given the file suffixes for that plot type
fn required_target_files(dir: &Path, target: &str, suffixes: &[&str]) -> Vec<PathBuf> {
    suffixes
        .iter()
        .map(|suffix| dir.join(format!("{target}{suffix}")))
        .collect()
}

/// Checks if the required files exist to create a coverage plot for the given
/// target, based on a Vec of theoretical paths
fn validate_target_files(target: &str, required_files: Vec<PathBuf>, plot_type: &str) -> bool {
    let mut missing_files = Vec::new();

    for path in required_files {
        if !path.is_file() {
            missing_files.push(path);
            continue;
        }
    }

    if missing_files.is_empty() {
        return true;
    }

    // The existence of clustermap matrices is dependent on the data, and it is
    // quite likely that for some segments there is a clustermap and some there
    // is not, even if clustermap is enabled for the entire run.
    if plot_type != "clustermap" {
        eprintln!(
            "Could not create {plot_type} plot for {target}; missing required files: {}",
            missing_files
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    false
}

fn is_valid_target_name(target: &str) -> bool {
    !target.is_empty()
        && target.len() <= 128
        && target
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
