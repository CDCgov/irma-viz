use serde_derive::Deserialize;
use clap::Parser;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub plots: PlotsConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlotsConfig {
    pub density_average: bool,
    pub density_8: bool,
    pub density_observed: bool,
    pub observed_8: bool,
    pub coverage: bool,
    pub confidence: bool,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: String,
    pub width: u32,
    pub height: u32,
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
    pub out: Option<String>,

    /// Output width override
    #[arg(long)]
    pub width: Option<u32>,

    /// Output height override
    #[arg(long)]
    pub height: Option<u32>,
    
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


#[derive(Debug, Parser)]
pub struct HeuristicsArgs {
    #[arg(long)]
    pub alleles_tsv: String,

    #[arg(long)]
    pub min_aq: f64,

    #[arg(long)]
    pub min_f: f64,

    #[arg(long)]
    pub min_tcc: f64,
    
    #[arg(long)]
    pub min_conf: f64,
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
    if let Some(w) = args.width {
        cfg.output.width = w;
    }
    if let Some(h) = args.height {
        cfg.output.height = h;
    }

    // plot overrides
    merge_plot_bool(&mut cfg.plots.density_average, args.enabled_plots.density_average);
    merge_plot_bool(&mut cfg.plots.density_8, args.enabled_plots.density_8);
    merge_plot_bool(&mut cfg.plots.density_observed, args.enabled_plots.density_observed);
    merge_plot_bool(&mut cfg.plots.observed_8, args.enabled_plots.observed_8);
    merge_plot_bool(&mut cfg.plots.coverage, args.enabled_plots.coverage);
    merge_plot_bool(&mut cfg.plots.confidence, args.enabled_plots.confidence);

    cfg
}


