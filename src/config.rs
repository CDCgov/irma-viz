use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::{fs, path::PathBuf};

// (I think) we will want to have separate configuration options for each plot

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
    pub heuristics_args: HeuristicsArgs,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub targets: Targets,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub plot_toggles: PlotToggles,
    pub constants: HeuristicsConfig,
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

// to-do: add subplot configs
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
pub struct HeuristicsArgs {
    #[arg(long)]
    pub min_aq: Option<f64>,
    #[arg(long)]
    pub min_f: Option<f64>,
    #[arg(long)]
    pub min_tcc: Option<f64>,
    #[arg(long)]
    pub min_conf: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct HeuristicsConfig {
    pub min_aq: f64,
    pub min_f: f64,
    pub min_tcc: f64,
    pub min_conf: f64,
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
    merge(&mut cfg.constants.min_aq, args.heuristics_args.min_aq);
    merge(&mut cfg.constants.min_f, args.heuristics_args.min_f);
    merge(&mut cfg.constants.min_conf, args.heuristics_args.min_conf);
    merge(&mut cfg.constants.min_tcc, args.heuristics_args.min_tcc);

    cfg
}

pub fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("Error reading \'{path}\'"))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("Error parsing \'{path}\'"))?;
    Ok(cfg)
}
