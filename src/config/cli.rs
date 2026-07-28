use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::config::parsed_config::IOConfig;

/// These are for overriding settings from the config.toml
#[derive(Debug, Parser)]
#[command(name = "irma-viz", version, about = "Render IRMA plots to SVG")]
pub struct CLIConfig {
    #[command(flatten)]
    pub io_args: IOArgsCLI,
    /// Path to config TOML
    #[arg(long, short = 'c', default_value = "config.toml")]
    pub config: String,
    /// Render one example of every plot type for a single explicit target.
    /// Only available when built with the `demo` feature.
    #[cfg(feature = "demo")]
    #[arg(long)]
    pub demo_target: Option<String>,
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
#[derive(Debug, Parser, Copy, Clone)]
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
