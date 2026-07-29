use std::fs;

use anyhow::{Context as _, Result};
use serde::Deserialize;

use crate::config::{
    OutputFormat,
    parsed_config::{ClusterConfig, CoverageConfig, PlotToggles},
};

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

/// Parses the `config.toml`` file into structs using the [toml] crate
pub fn load_config(path: &str) -> Result<TOMLConfig> {
    let s = fs::read_to_string(path).with_context(|| format!("Error reading \'{path}\'"))?;
    let cfg: TOMLConfig =
        toml::from_str(&s).with_context(|| format!("Error parsing \'{path}\'"))?;
    Ok(cfg)
}

/// Plot specific Options within the TOML
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

/// All configuration options for read-percent plots
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct ReadPercentTOML {
    pub viz_option: PercentVizOptionTOML,
}

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

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct HeuristicsPlots {
    pub allele_quality: bool,
    pub quality_subplot: bool,
    pub allele_frequency: bool,
    pub frequency_subplot: bool,
    pub coverage_depth_hist: bool,
    pub confidence_hist: bool,
}

impl HeuristicsPlots {
    pub fn check_any_enabled(&self) -> bool {
        self.allele_quality
            || self.quality_subplot
            || self.allele_frequency
            || self.frequency_subplot
            || self.coverage_depth_hist
            || self.confidence_hist
    }
}
