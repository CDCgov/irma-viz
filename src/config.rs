use clap::Parser;
use kuva::plot::LineStyle;
use serde_derive::Deserialize;
use std::path::PathBuf;

/// (I think) we will want to have separate configuration options for each plot

#[derive(Debug, Deserialize)]
pub struct Config {
    pub plots: PlotsConfig,
    pub output: OutputConfig,
    pub constants: HeuristicsArgs,
}

#[derive(Debug, Deserialize)]
pub struct PlotsConfig {
    pub heuristics_path: Option<PathBuf>,
    pub density_average: bool,
    pub density_8: bool,
    pub density_observed: bool,
    pub observed_8: bool,
    pub coverage: bool,
    pub confidence: bool,
    pub sankey_path: Option<PathBuf>,
    pub coverage_path: Option<PathBuf>,
    pub pairing_stats_path: Option<PathBuf>,
    pub variants_path: Option<PathBuf>,
    pub sqm_path: Option<PathBuf>,
}

// to-do: add subplot configs
#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: PathBuf,
}

/// These are for overriding settings from the config.toml
#[derive(Debug, Parser)]
#[command(name = "irma-viz", version, about = "Render IRMA plots to SVG")]
pub struct PlottingArgs {
    /// Path to config TOML
    #[arg(long, default_value = "config.toml")]
    pub config: String,

    /// Output SVG path override
    #[arg(long)]
    pub out: Option<PathBuf>,

    #[command(flatten)]
    pub enabled_plots: PlotToggles,
    #[command(flatten)]
    pub heuristics_args: HeuristicsArgs,
}

// toggles for enabling/disabling to override the config
// if these flags aren't used, the default will stick
// e.g.
//   `--density-average true`
#[derive(Debug, Parser)]
pub struct PlotToggles {
    #[arg(long)]
    pub density_average: Option<bool>,

    #[arg(long)]
    pub density_8: Option<bool>,

    #[arg(long)]
    pub density_observed: Option<bool>,

    #[arg(long)]
    pub observed_8: Option<bool>,

    #[arg(long)]
    pub coverage: Option<bool>,

    #[arg(long)]
    pub confidence: Option<bool>,
}

#[derive(Debug, Parser, Deserialize)]
pub struct HeuristicsArgs {
    #[arg(long)]
    pub heuristics_path: Option<PathBuf>,

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

pub fn apply_cli_overrides(mut cfg: Config, args: &PlottingArgs) -> Config {
    // output overrides
    if let Some(out) = &args.out {
        cfg.output.path = out.clone();
    }

    // plot overrides
    merge_plot_bool(
        &mut cfg.plots.density_average,
        args.enabled_plots.density_average,
    );
    merge_plot_bool(&mut cfg.plots.density_8, args.enabled_plots.density_8);
    merge_plot_bool(
        &mut cfg.plots.density_observed,
        args.enabled_plots.density_observed,
    );
    merge_plot_bool(&mut cfg.plots.observed_8, args.enabled_plots.observed_8);
    merge_plot_bool(&mut cfg.plots.coverage, args.enabled_plots.coverage);
    merge_plot_bool(&mut cfg.plots.confidence, args.enabled_plots.confidence);

    cfg
}

#[allow(unused)]
#[derive(Debug)]
pub struct LinePlotConfig {
    pub color: String,
    pub stroke_width: f32,
    pub title: String,
    pub xlabel: String,
    pub ylabel: String,
    pub line_style: LineStyle,
}
