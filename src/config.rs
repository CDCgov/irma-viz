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

const TABLE_TARGET_SUFFIXES: &[&str] = &[
    "-allAlleles.txt",
    "-coverage.txt",
    "-pairingStats.txt",
    "-variants.txt",
    "-insertions.txt",
    "-deletions.txt",
];
const MATRIX_TARGET_SUFFIXES: &[&str] = &[
    "-EXPENRD.sqm",
    "-JACCARD.sqm",
    "-MUTUALD.sqm",
    "-NJOINTP.sqm",
];

/// takes the input root for an IRMA run and returns paths for the `tables/` and `matrices/` directories
pub fn get_directory_paths(input_root: &Path) -> (PathBuf, PathBuf) {
    (input_root.join("tables"), input_root.join("matrices"))
}

pub fn resolve_targets(cfg: &Config) -> Result<Vec<String>> {
    if !(cfg.plot_toggles.heuristics || cfg.plot_toggles.coverage || cfg.plot_toggles.clustermap) {
        return Ok(Vec::new());
    }

    let mut discovered = BTreeSet::new();
    let io_args = cfg.io_args()?;
    let (table_path, matrix_path) = get_directory_paths(&io_args.input_root);

    discovered.extend(discover_targets_in_dir(&table_path, TABLE_TARGET_SUFFIXES)?);
    if cfg.plot_toggles.clustermap {
        discovered.extend(discover_targets_in_dir(
            &matrix_path,
            MATRIX_TARGET_SUFFIXES,
        )?);
    }

    if discovered.is_empty()
        && (cfg.plot_toggles.heuristics || cfg.plot_toggles.coverage || cfg.plot_toggles.clustermap)
    {
        return Err(anyhow::anyhow!(
            "No targets were discovered under '{}' and '{}'",
            table_path.display(),
            matrix_path.display()
        ));
    }

    Ok(discovered.into_iter().collect())
}

fn discover_targets_in_dir(dir: &Path, suffixes: &[&str]) -> Result<BTreeSet<String>> {
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
            if let Some(target) = file_name.strip_suffix(suffix)
                && !target.is_empty()
            {
                targets.insert(target.to_owned());
                break;
            }
        }
    }

    Ok(targets)
}
