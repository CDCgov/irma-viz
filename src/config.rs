use anyhow::Context;
use clap::Parser;
use serde_derive::Deserialize;
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
    /// Output SVG path override
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Which figures to plot
    #[command(flatten)]
    pub enabled_plots: PlotToggleArgs,
    /// Constants for heuristic plot
    #[command(flatten)]
    pub heuristics_args: HeuristicsArgs,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub targets: Targets,
    pub input: InputConfig,
    pub output: OutputConfig,
    pub plot_toggles: PlotToggles,
    pub constants: HeuristicsArgs,
}

#[derive(Debug, Deserialize)]
pub struct Targets {
    pub list: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlotToggles {
    pub read_percentages: bool,
    pub heuristics: bool,
    pub coverage: bool,
    pub clustermap: bool,
}

#[derive(Debug, Deserialize)]
pub struct InputConfig {
    pub data_path: PathBuf,
    pub matrix_path: PathBuf,
}

// to-do: add subplot configs
#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: PathBuf,
}

// toggles for enabling/disabling to override the config
// if these flags aren't used, the default will stick
// e.g.
//   `--density-average true`
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

fn merge_plot_bool(target: &mut bool, override_val: Option<bool>) {
    if let Some(v) = override_val {
        *target = v;
    }
}

pub fn apply_cli_overrides(mut cfg: Config, args: &Args) -> Config {
    // output overrides
    if let Some(out) = &args.out {
        cfg.output.path = out.clone();
    }

    // plot overrides
    merge_plot_bool(
        &mut cfg.plot_toggles.read_percentages,
        args.enabled_plots.read_percentages,
    );
    merge_plot_bool(
        &mut cfg.plot_toggles.heuristics,
        args.enabled_plots.heuristics,
    );
    merge_plot_bool(&mut cfg.plot_toggles.coverage, args.enabled_plots.coverage);
    merge_plot_bool(
        &mut cfg.plot_toggles.clustermap,
        args.enabled_plots.clustermap,
    );

    cfg
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("Error reading /'{path}/'"))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("Error parsing /'{path}/'"))?;
    Ok(cfg)
}
