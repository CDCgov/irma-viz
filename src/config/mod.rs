mod cli;
mod matrices;
mod parsed_config;
mod targets;
mod toml;

pub use cli::CLIConfig;
pub use matrices::MatrixType;
pub use parsed_config::{
    ClusterOption, CoverageColorOption, HeuristicsConfig, OutputFormat, ParsedConfig,
    PercentVizOption,
};
#[cfg(feature = "demo")]
pub use targets::is_valid_target_name;
pub use targets::{
    ClusterTargets, discover_clustermap_targets, discover_coverage_targets,
    discover_heuristics_targets,
};
pub use toml::load_config;
