mod cli;
mod matrices;
mod parsed_config;
mod targets;
mod toml;

pub use cli::CLIConfig;
pub use parsed_config::{
    ClusterOption, CoverageColorOption, HeuristicsConfig, OutputFormat, ParsedConfig,
    PercentVizOption, get_directory_paths,
};
pub use toml::load_config;
#[cfg(feature = "demo")]
pub use {matrices::MatrixType, targets::is_valid_target_name};
