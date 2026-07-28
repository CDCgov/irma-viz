use anyhow::{Context, Result};
use clap::{ArgAction, Parser, ValueEnum};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// These are for overriding settings from the config.toml
#[derive(Debug, Parser)]
#[command(name = "irma-viz", version, about = "Render IRMA plots to SVG")]
pub struct CLIConfig {
    #[command(flatten)]
    pub io_args: IOArgsCLI,
    /// Path to config TOML
    #[arg(long, short = 'c', default_value = "config.toml")]
    pub config: String,
    /// Which figures to plot
    #[command(flatten)]
    pub enabled_plots: PlotToggleCLI,
    /// Plot specific args
    #[command(flatten)]
    pub plot_specific_args: PlotSpecificCLI,
}

#[derive(Debug, Parser, Clone)]
pub struct IOArgsCLI {
    /// Path to input directory that contains `tables/` and `matrices/`
    #[arg(long, short = 'i')]
    pub input_root: PathBuf,
    /// Destination directory for output figures. If not specified, defaults to
    /// `input_root/figures/`
    #[arg(long, short = 'o')]
    pub output_path: Option<PathBuf>,
}

impl IOArgsCLI {
    /// Parses IO args by setting `output_path` to `input_root/figures` if no
    /// `output_path` is otherwise specified
    pub fn parse_io_args(self) -> IOConfig {
        let IOArgsCLI {
            input_root,
            output_path,
        } = self;
        let output_path = output_path
            .clone()
            .unwrap_or_else(|| input_root.join("figures"));
        IOConfig {
            input_root,
            output_path,
        }
    }
}

/// A parsed IO Config
#[derive(Debug)]
pub struct IOConfig {
    /// The path to the root directory of an IRMA run, which expects a `tables`
    /// and `matrices` directory within
    pub input_root: PathBuf,
    /// Path to the destination directory for created plots
    pub output_path: PathBuf,
}

/// Plot toggles, both from the TOML and the parsed Config
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct PlotToggles {
    pub read_percentages: bool,
    pub heuristics: bool,
    pub coverage: bool,
    pub clustermap: bool,
}

/// helper function for overriding TOML values with CLI values, if they exist
fn merge_toml_cli_value<T>(toml_val: T, cli_val: Option<T>) -> T {
    let mut result = toml_val;
    if let Some(v) = cli_val {
        result = v;
    }
    result
}

impl PlotToggles {
    /// helper function for overriding TOML options with CLI options, if
    /// applicable
    pub fn merge_plot_toggles(toml: PlotToggles, cli: PlotToggleCLI) -> PlotToggles {
        let read_percentages = merge_toml_cli_value(toml.read_percentages, cli.read_percentages);
        let heuristics = merge_toml_cli_value(toml.heuristics, cli.heuristics);
        let coverage = merge_toml_cli_value(toml.coverage, cli.coverage);
        let clustermap = merge_toml_cli_value(toml.clustermap, cli.clustermap);

        PlotToggles {
            read_percentages,
            heuristics,
            coverage,
            clustermap,
        }
    }
}

/// Toggles for enabling/disabling plot types within the CLI; overrides TOML
/// options
//   e.g. `--coverage true`
#[derive(Debug, Parser, Copy, Clone)]
pub struct PlotToggleCLI {
    #[arg(long)]
    pub read_percentages: Option<bool>,
    #[arg(long)]
    pub heuristics: Option<bool>,
    #[arg(long)]
    pub coverage: Option<bool>,
    #[arg(long)]
    pub clustermap: Option<bool>,
}

/// Threshold values for heuristics plots to be passed via CLI
#[derive(Debug, Parser, Deserialize, Copy, Clone)]
pub struct HeuristicsConfig {
    /// Minimum average allele quality score heuristic for calling insertion &
    /// single nucleotide variants
    #[arg(long, default_value_t = 24.0)]
    pub min_aq: f64,
    /// Minimum frequency heuristic for calling single nucleotide variants
    #[arg(long, default_value_t = 0.008)]
    pub min_f: f64,
    /// Minimum coverage depth heuristic (total coverage count) for calling
    /// variants
    #[arg(long, default_value_t = 100.0)]
    pub min_tcc: f64,
    /// Minimum confidence not machine error for single nucleotide variants
    #[arg(long, default_value_t = 0.8)]
    pub min_conf: f64,
}

/// Plot specific Options within the TOML
#[derive(Debug, Deserialize)]
pub struct PlotSpecificTOML {
    #[serde(rename = "coverage_options")]
    pub coverage: CoverageConfig,

    #[serde(rename = "percent_options")]
    pub read_percent: ReadPercentTOML,

    #[serde(rename = "cluster_options")]
    pub cluster_config: ClusterConfig,
}

/// Plot specific optoins within the CLI
#[derive(Debug, Parser)]
pub struct PlotSpecificCLI {
    /// Whether the input reads are in a paired `fastq` format
    #[arg(long, action = ArgAction::Set, required = true)]
    pub paired: Option<bool>,

    /// Tree height for agglomerative clustering
    #[arg(long)]
    pub tree_height: Option<f64>,

    #[command(flatten)]
    pub heuristics_args: HeuristicsConfig,
}

/// Controls whether coverage variant reference lines are colored by ACGT
/// identity of the variant nucleotide, or by the observed frequency of the
/// variant
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum CoverageColorOption {
    Nucleotide,
    Frequency,
}

/// For selecting between a sankey flow diagram and a dashboard of pie charts
/// describing the classifications of the reads in the IRMA run
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum PercentVizOption {
    Sankey,
    Pie,
}

/// Selects for clustermap plot whether to use a plain heatmap or a phylogenetic
/// tree paired with a heatmap
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ClusterOption {
    Clustermap,
    Tree,
}

/// Holds all config options for coverage plot
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CoverageConfig {
    #[serde(rename = "variant_color")]
    pub color_option: CoverageColorOption,
}

/// Holds all config options for cluster plots, both in TOML and parsed forms
#[derive(Debug, Deserialize)]
pub struct ClusterConfig {
    pub cluster_option: ClusterOption,
    pub matrix_types: MatrixTypes,
    pub tree_height: f64,
}

/// Possible matrix types for cluster plot input/output
#[derive(Debug, Deserialize, Clone, Copy)]
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

impl MatrixTypes {
    /// Converts the struct of bools created from the config.toml into a Vec of
    /// enabled matrix types for iteration
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
    /// for generating output filenames for the cluster plots
    pub fn display_name(self) -> &'static str {
        match self {
            MatrixType::Expenrd => "EXPENRD",
            MatrixType::Jaccard => "JACCARD",
            MatrixType::Mutuald => "MUTUALD",
            MatrixType::Njointp => "NJOINTP",
        }
    }

    /// for generating filenames for reading in cluster matrix data
    pub fn file_suffix(self) -> &'static str {
        match self {
            MatrixType::Expenrd => "-EXPENRD.sqm",
            MatrixType::Jaccard => "-JACCARD.sqm",
            MatrixType::Mutuald => "-MUTUALD.sqm",
            MatrixType::Njointp => "-NJOINTP.sqm",
        }
    }
}

/// All configuration options for read-percent plots
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct ReadPercentTOML {
    pub viz_option: PercentVizOption,
}

/// Parsed read-percent plot options
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ReadPercentConfig {
    pub viz_option: PercentVizOption,
    pub paired: bool,
}

/// Parses the `config.toml`` file into structs using the [toml] crate
pub fn load_config(path: &str) -> Result<TOMLConfig> {
    let s = fs::read_to_string(path).with_context(|| format!("Error reading \'{path}\'"))?;
    let cfg: TOMLConfig =
        toml::from_str(&s).with_context(|| format!("Error parsing \'{path}\'"))?;
    Ok(cfg)
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
    pub fn check_missing_targets(&self, plot_toggles: &PlotToggles, matrix_types: &MatrixTypes) {
        if plot_toggles.heuristics && self.heuristics.is_empty() {
            eprintln!("Warning: heuristics plotting was enabled but no valid targets were found");
        }
        if plot_toggles.coverage && self.coverage.is_empty() {
            eprintln!("Warning: coverage plotting was enabled but not valid targets were found");
        }

        if plot_toggles.heuristics && plot_toggles.coverage {
            warn_missing(&self.heuristics, &self.coverage, "heuristics", "coverage");
            warn_missing(&self.coverage, &self.heuristics, "coverage", "heuristics");
        }

        let clustermap_targets = self.clustermap.variant_targets();

        if plot_toggles.heuristics && plot_toggles.clustermap {
            warn_missing(
                &clustermap_targets,
                &self.heuristics,
                "clustermap",
                "heuristics",
            );
        }

        if plot_toggles.coverage && plot_toggles.clustermap {
            warn_missing(
                &clustermap_targets,
                &self.coverage,
                "clustermap",
                "coverage",
            );
        }

        if plot_toggles.clustermap {
            self.clustermap.check_missing_matrix_targets(matrix_types);
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

/// get path to `tables` and `matrices` directories before calling to
/// [discover_targets_by_plot_type]
pub fn resolve_targets(
    plot_toggles: &PlotToggles,
    io_args: &IOConfig,
    matrix_types: &MatrixTypes,
) -> Result<PlotTargets> {
    // no plots needing targets enabled
    if !(plot_toggles.heuristics || plot_toggles.coverage || plot_toggles.clustermap) {
        return Ok(PlotTargets::default());
    }

    let (table_path, matrix_path) = get_directory_paths(&io_args.input_root);
    discover_targets_by_plot_type(&table_path, &matrix_path, plot_toggles, matrix_types)
}

/// finds all valid targets for each given plot type and stores them seperately
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

/// Takes a path to a directory and a list of suffixes and returns a BTreeSet
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TOMLConfig {
    pub plot_toggles: PlotToggles,

    #[serde(flatten)]
    pub plot_specific: PlotSpecificTOML,
}

/// Parsed and merged plot-specific options
#[derive(Debug)]
pub struct PlotSpecificConfig {
    pub coverage: CoverageConfig,
    pub read_percent: ReadPercentConfig,
    pub cluster_config: ClusterConfig,
    pub heuristic: HeuristicsConfig,
}

fn validate_heuristics_thresholds(
    heuristics: HeuristicsConfig,
    toggles: PlotToggles,
) -> Result<()> {
    if !toggles.heuristics {
        // don't need to bother validating, no heuristics plot being created
        return Ok(());
    }

    validate_finite_range("min_f", heuristics.min_f, 0.0, 1.0)?;

    validate_finite_range("min_conf", heuristics.min_conf, 0.0, 1.0)?;

    validate_finite_range("min_aq", heuristics.min_aq, 0.0, 64.0)?;

    if !heuristics.min_tcc.is_finite() || heuristics.min_tcc < 1.0 {
        anyhow::bail!(
            "Error: Value min_tcc must be finite and greater than or equal to 1, {} was provided",
            heuristics.min_tcc
        );
    }

    Ok(())
}

fn validate_finite_range(name: &str, value: f64, min: f64, max: f64) -> Result<()> {
    if !value.is_finite() || value < min || value > max {
        anyhow::bail!(
            "Value {name} must be finite and between {min} and {max}, {value} was provided"
        );
    }
    Ok(())
}

impl PlotSpecificConfig {
    fn merge_plot_specifics(
        toml: PlotSpecificTOML,
        cli: PlotSpecificCLI,
        toggles: PlotToggles,
    ) -> Result<Self> {
        // coverage options are only provided via TOML
        let coverage = toml.coverage;

        // heuristics options are only provided via CLI
        let heuristic = cli.heuristics_args;
        validate_heuristics_thresholds(heuristic, toggles)?;

        let read_percent = ReadPercentConfig {
            // viz option is provided via TOML
            viz_option: toml.read_percent.viz_option,
            // paired is provided via CLI. we can unwrap here because Clap
            // guarantees the argument is present
            paired: cli.paired.unwrap(),
        };
        // = toml.plot_specific.read_percent;

        let cluster_config = ClusterConfig {
            // cluster option is provided via TOML
            cluster_option: toml.cluster_config.cluster_option,
            // matrix types are provided via TOML
            matrix_types: toml.cluster_config.matrix_types,
            // tree height comes from TOML + CLI
            tree_height: merge_toml_cli_value(toml.cluster_config.tree_height, cli.tree_height),
        };

        validate_finite_range("tree_height", cluster_config.tree_height, 0.0, 1.0)?;

        Ok(PlotSpecificConfig {
            coverage,
            read_percent,
            cluster_config,
            heuristic,
        })
    }
}

#[derive(Debug)]
pub struct ParsedConfig {
    pub plot_toggles: PlotToggles,
    pub io_args: IOConfig,
    pub plot_targets: PlotTargets,
    pub plot_specific: PlotSpecificConfig,
}

impl ParsedConfig {
    pub fn merge_configs(toml: TOMLConfig, cli: CLIConfig) -> Result<Self> {
        let io_args = cli.io_args.parse_io_args();

        let plot_toggles = PlotToggles::merge_plot_toggles(toml.plot_toggles, cli.enabled_plots);

        let plot_specific = PlotSpecificConfig::merge_plot_specifics(
            toml.plot_specific,
            cli.plot_specific_args,
            plot_toggles,
        )?;

        // get valid plot targets
        let plot_targets = resolve_targets(
            &plot_toggles,
            &io_args,
            &plot_specific.cluster_config.matrix_types,
        )?;
        // check and warn of missing targets based on enabled plots and discovered targets
        plot_targets
            .check_missing_targets(&plot_toggles, &plot_specific.cluster_config.matrix_types);

        Ok(ParsedConfig {
            plot_toggles,
            io_args,
            plot_targets,
            plot_specific,
        })
    }
}
