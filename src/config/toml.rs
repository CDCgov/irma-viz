use std::fs;

use anyhow::{Context as _, Result};
use serde::Deserialize;

use crate::config::{
    OutputFormat,
    parsed_config::{ClusterConfig, CoverageConfig, PercentVizOption, PlotToggles},
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
    pub viz_option: PercentVizOption,
}
