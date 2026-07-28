use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

use crate::config::{
    cli::{CLIConfig, HeuristicsConfig, PlotSpecificCLI, PlotToggleCLI},
    matrices::MatrixTypes,
    targets::{PlotTargets, resolve_targets},
    toml::{PlotSpecificTOML, TOMLConfig},
};

/// A parsed IO Config
#[derive(Debug)]
pub struct IOConfig {
    /// The path to the root directory of an IRMA run, which expects a `tables`
    /// and `matrices` directory within
    pub input_root: PathBuf,
    /// Path to the destination directory for created plots
    pub output_path: PathBuf,
}

/// Plot toggles, both from the TOML and the parsed Config
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct PlotToggles {
    pub read_percentages: bool,
    pub heuristics: bool,
    pub coverage: bool,
    pub clustermap: bool,
}

/// helper function for overriding TOML values with CLI values, if they exist
fn merge_toml_cli_value<T>(toml_val: T, cli_val: Option<T>) -> T {
    let mut result = toml_val;
    if let Some(v) = cli_val {
        result = v;
    }
    result
}

impl PlotToggles {
    /// helper function for overriding TOML options with CLI options, if
    /// applicable
    pub fn merge_plot_toggles(toml: PlotToggles, cli: PlotToggleCLI) -> PlotToggles {
        let read_percentages = merge_toml_cli_value(toml.read_percentages, cli.read_percentages);
        let heuristics = merge_toml_cli_value(toml.heuristics, cli.heuristics);
        let coverage = merge_toml_cli_value(toml.coverage, cli.coverage);
        let clustermap = merge_toml_cli_value(toml.clustermap, cli.clustermap);

        PlotToggles {
            read_percentages,
            heuristics,
            coverage,
            clustermap,
        }
    }
}

/// Holds all config options for coverage plot
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CoverageConfig {
    #[serde(rename = "variant_color")]
    pub color_option: CoverageColorOption,
}

/// Holds all config options for cluster plots, both in TOML and parsed forms
#[derive(Debug, Deserialize)]
pub struct ClusterConfig {
    pub cluster_option: ClusterOption,
    pub matrix_types: MatrixTypes,
    pub tree_height: f64,
}

/// Controls whether coverage variant reference lines are colored by ACGT
/// identity of the variant nucleotide, or by the observed frequency of the
/// variant
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CoverageColorOption {
    Nucleotide,
    Frequency,
}

/// For selecting between a sankey flow diagram and a dashboard of pie charts
/// describing the classifications of the reads in the IRMA run
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PercentVizOption {
    Sankey,
    Pie,
}

/// Selects for clustermap plot whether to use a plain heatmap or a phylogenetic
/// tree paired with a heatmap
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ClusterOption {
    Clustermap,
    Tree,
}

/// Parsed read-percent plot options
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ReadPercentConfig {
    pub viz_option: PercentVizOption,
    pub paired: bool,
}

/// takes the input root for an IRMA run and returns paths for the `tables/` and `matrices/` directories
pub fn get_directory_paths(input_root: &Path) -> (PathBuf, PathBuf) {
    (input_root.join("tables"), input_root.join("matrices"))
}

/// Parsed and merged plot-specific options
#[derive(Debug)]
pub struct PlotSpecificConfig {
    pub coverage: CoverageConfig,
    pub read_percent: ReadPercentConfig,
    pub cluster_config: ClusterConfig,
    pub heuristic: HeuristicsConfig,
}

fn validate_heuristics_thresholds(
    heuristics: HeuristicsConfig,
    toggles: PlotToggles,
) -> Result<()> {
    if !toggles.heuristics {
        // don't need to bother validating, no heuristics plot being created
        return Ok(());
    }

    validate_finite_range("min_f", heuristics.min_f, 0.0, 1.0)?;

    validate_finite_range("min_conf", heuristics.min_conf, 0.0, 1.0)?;

    validate_finite_range("min_aq", heuristics.min_aq, 0.0, 64.0)?;

    if !heuristics.min_tcc.is_finite() || heuristics.min_tcc < 1.0 {
        anyhow::bail!(
            "Error: Value min_tcc must be finite and greater than or equal to 1, {} was provided",
            heuristics.min_tcc
        );
    }

    Ok(())
}

fn validate_finite_range(name: &str, value: f64, min: f64, max: f64) -> Result<()> {
    if !value.is_finite() || value < min || value > max {
        anyhow::bail!(
            "Value {name} must be finite and between {min} and {max}, {value} was provided"
        );
    }
    Ok(())
}

impl PlotSpecificConfig {
    fn merge_plot_specifics(
        toml: PlotSpecificTOML,
        cli: PlotSpecificCLI,
        toggles: PlotToggles,
    ) -> Result<Self> {
        // coverage options are only provided via TOML
        let coverage = toml.coverage;

        // heuristics options are only provided via CLI
        let heuristic = cli.heuristics_args;
        validate_heuristics_thresholds(heuristic, toggles)?;

        let read_percent = ReadPercentConfig {
            // viz option is provided via TOML
            viz_option: toml.read_percent.viz_option,
            // paired is provided via CLI. we can unwrap here because Clap
            // guarantees the argument is present
            paired: cli.paired.unwrap(),
        };

        let cluster_config = ClusterConfig {
            // cluster option is provided via TOML
            cluster_option: toml.cluster_config.cluster_option,
            // matrix types are provided via TOML
            matrix_types: toml.cluster_config.matrix_types,
            // tree height comes from TOML + CLI
            tree_height: merge_toml_cli_value(toml.cluster_config.tree_height, cli.tree_height),
        };

        validate_finite_range("tree_height", cluster_config.tree_height, 0.0, 1.0)?;

        Ok(PlotSpecificConfig {
            coverage,
            read_percent,
            cluster_config,
            heuristic,
        })
    }
}

#[derive(Debug)]
pub struct ParsedConfig {
    pub plot_toggles: PlotToggles,
    pub io_args: IOConfig,
    pub plot_targets: PlotTargets,
    pub plot_specific: PlotSpecificConfig,
}

impl ParsedConfig {
    pub fn merge_configs(toml: TOMLConfig, cli: CLIConfig) -> Result<Self> {
        let io_args = cli.io_args.parse_io_args();

        let plot_toggles = PlotToggles::merge_plot_toggles(toml.plot_toggles, cli.enabled_plots);

        let plot_specific = PlotSpecificConfig::merge_plot_specifics(
            toml.plot_specific,
            cli.plot_specific_args,
            plot_toggles,
        )?;

        // get valid plot targets
        let plot_targets = resolve_targets(
            &plot_toggles,
            &io_args,
            &plot_specific.cluster_config.matrix_types,
        )?;
        // check and warn of missing targets based on enabled plots and discovered targets
        plot_targets
            .check_missing_targets(&plot_toggles, &plot_specific.cluster_config.matrix_types);

        Ok(ParsedConfig {
            plot_toggles,
            io_args,
            plot_targets,
            plot_specific,
        })
    }
}
