use std::path::PathBuf;

use clap::Parser;

/// Command-line configuration for the `irma-viz` executable.
#[derive(Debug, Parser)]
#[command(name = "irma-viz", version, about = "Render IRMA plots")]
pub struct CLIConfig {
    #[command(flatten)]
    pub io_args: IOArgsCLI,
    /// Path to config TOML
    #[arg(long, short = 'c', default_value = "irma-viz-config.toml")]
    pub config: String,
    /// Render the svg demo set for a single explicit target. Only available
    /// when built with the `demo` feature.
    #[cfg(feature = "demo")]
    #[arg(long)]
    pub demo_target: Option<String>,
    /// CLI overrides for configured plot toggles.
    #[command(flatten)]
    pub enabled_plots: PlotToggleCLI,
    /// Additional CLI arguments for specific plot types
    #[command(flatten)]
    pub plot_specific_args: PlotSpecificCLI,
}

/// Input and output paths supplied on the command line
#[derive(Debug, Parser, Clone)]
pub struct IOArgsCLI {
    /// Path to the IRMA run directory containing `tables/` and `matrices/`.
    #[arg(long, short = 'i')]
    pub input_root: PathBuf,
    /// Destination directory for output figures. If omitted, defaults to
    /// `input_root/figures/`
    #[arg(long, short = 'o')]
    pub output_path: Option<PathBuf>,
}

/// Optional command-line overrides for the TOML plot toggles.
///
/// An absent value preserves the configured value; for example, `--coverage
/// true` enables coverage plots even if TOML disables them.
#[derive(Debug, Parser, Copy, Clone)]
pub struct PlotToggleCLI {
    /// Overrides the `read_percentages` toggle
    #[arg(long)]
    pub read_percentages: Option<bool>,
    /// Overrides the `heuristics` toggle
    #[arg(long)]
    pub heuristics: Option<bool>,
    /// Overrides the `coverage` toggle
    #[arg(long)]
    pub coverage: Option<bool>,
    /// Overrides the `clustermap` toggle
    #[arg(long)]
    pub clustermap: Option<bool>,
}

/// Heuristic reference threshold values for heuristics plots
#[derive(Debug, Parser, Copy, Clone)]
pub struct HeuristicsCLI {
    /// Average-allele-quality reference threshold shown in heuristics plots.
    /// [0.0, 64.0]
    #[arg(long, default_value_t = 24.0)]
    pub min_variant_average_quality: f64,
    /// Variant-frequency reference threshold shown in heuristics plots. [0.0,
    /// 1.0]
    #[arg(long, default_value_t = 0.008)]
    pub min_variant_frequency: f64,
    /// Coverage-depth reference threshold shown in heuristics plots. (≥ 1)
    #[arg(long, default_value_t = 100.0)]
    pub min_variant_depth: f64,
    /// Minimum confidence-not-machine-error for single nucleotide variants
    /// shown in heuristics plots. [0.0, 1.0]
    #[arg(long, default_value_t = 0.8)]
    pub min_confidence_not_sequencer_error: f64,
}

/// Command-line arguments specific to individual plot types.
#[derive(Debug, Parser)]
pub struct PlotSpecificCLI {
    /// Whether input reads are in paired `fastq` format. Required for pie-style
    /// read-percentage plotting.
    #[arg(long)]
    pub paired: Option<bool>,

    /// Overrides `tree_height`, which positions the displayed
    /// dendrogram cutoff line in tree-style clustermaps. [0.0, 1.0]
    #[arg(long)]
    pub tree_height: Option<f64>,

    /// Plot-specific thresholds for heuristics plots
    #[command(flatten)]
    pub heuristics_args: HeuristicsCLI,
}
