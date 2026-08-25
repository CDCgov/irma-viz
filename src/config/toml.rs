//! TOML configuration schema and loading helpers.

use std::fs;

use serde::Deserialize;

use crate::{
    config::parsed_config::{ClusterConfig, CoverageConfig, OutputFormat, PlotToggles},
    diagnostics::PlotError,
};

/// Top-level `irma-viz` TOML configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TOMLConfig {
    pub output_options: OutputOptions,

    pub plot_toggles: PlotToggles,

    #[serde(flatten)]
    pub plot_specific: PlotSpecificTOML,
}

/// Output configuration loaded from the `[output_options]` TOML table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputOptions {
    pub output_format: OutputFormat,
}

/// Reads and deserializes the `irma-viz-config.toml` file into structs using
/// the [toml] crate
///
/// ## Errors
///
/// Passes up an error if the file cannot be read, or the file cannot be
/// deserialized   
pub fn load_config(path: &str) -> Result<TOMLConfig, PlotError> {
    let s = fs::read_to_string(path)
        .map_err(|err| PlotError::IOError(format!("reading '{path}'"), err))?;
    let cfg: TOMLConfig =
        toml::from_str(&s).map_err(|err| PlotError::InvalidData(err.to_string()))?;
    Ok(cfg)
}

/// Plot-specific TOML configuration tables.
#[derive(Debug, Deserialize)]
pub struct PlotSpecificTOML {
    #[serde(rename = "heuristics_options")]
    pub heuristics: HeuristicsTOML,

    #[serde(rename = "coverage_options")]
    pub coverage: CoverageConfig,

    #[serde(rename = "percent_options")]
    pub read_percent: ReadPercentTOML,

    #[serde(rename = "cluster_options")]
    pub cluster_config: ClusterConfig,
}

/// All configuration options for read-percent plots.
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct ReadPercentTOML {
    pub viz_option: PercentVizOptionTOML,
}

/// All configuration options for heuristics plots.
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct HeuristicsTOML {
    pub enabled_plots: HeuristicsPlots,
}

/// For selecting between a sankey flow diagram and a dashboard of pie charts
/// describing the classifications of the reads in the IRMA run
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PercentVizOptionTOML {
    Sankey,
    Pie,
}

/// Enables the individual panels in a heuristics figure.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct HeuristicsPlots {
    /// Full-range average-allele-quality density panel.
    pub allele_quality: bool,
    /// Threshold-focused average-allele-quality density panel.
    pub quality_subplot: bool,
    /// Full-range minority-allele-frequency density panel.
    pub allele_frequency: bool,
    /// Threshold-focused minority-allele-frequency density panel.
    pub frequency_subplot: bool,
    /// Low-depth coverage histogram panel.
    pub coverage_depth_hist: bool,
    /// Positive confidence-not-machine-error histogram panel.
    pub confidence_hist: bool,
}

impl HeuristicsPlots {
    /// Returns whether at least one heuristics panel is enabled.
    pub fn check_any_enabled(&self) -> bool {
        self.allele_quality
            || self.quality_subplot
            || self.allele_frequency
            || self.frequency_subplot
            || self.coverage_depth_hist
            || self.confidence_hist
    }
}
