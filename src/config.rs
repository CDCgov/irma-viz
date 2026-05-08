use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::{fs, path::PathBuf};

/// These are for overriding settings from the config.toml
#[derive(Debug, Parser)]
#[command(name = "irma-viz", version, about = "Render IRMA plots to SVG")]
pub struct Args {
    /// Path to config TOML
    #[arg(long, default_value = "config.toml")]
    pub config: String,
    /// Target organisms to plot
    #[arg(short, long)]
    pub targets: Vec<String>,
    /// Path where the input data tables are held
    #[arg(long)]
    pub table_path: Option<PathBuf>,
    /// Path where the input matrices are held
    #[arg(long)]
    pub matrix_path: Option<PathBuf>,
    /// Output SVG path override
    #[arg(long)]
    pub output: Option<PathBuf>,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub targets: Targets,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub plot_toggles: PlotToggles,
    pub constants: ConstantsConfig,

    #[serde(flatten)]
    pub plot_specific: PlotSpecificConfig,
}

#[derive(Debug, Deserialize)]
pub struct Targets {
    pub list: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct InputConfig {
    pub table_path: PathBuf,
    pub matrix_path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: PathBuf,
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
    Heatmap,
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
    // targets overrides
    if !args.targets.is_empty() {
        cfg.targets.list = args.targets.clone();
    }

    // input overrides
    if let Some(table_path) = &args.table_path {
        cfg.input.table_path = table_path.to_path_buf();
    }
    if let Some(matrix_path) = &args.matrix_path {
        cfg.input.matrix_path = matrix_path.to_path_buf();
    }

    // output overrides
    if let Some(out) = &args.output {
        cfg.output.path = out.clone();
    }

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
