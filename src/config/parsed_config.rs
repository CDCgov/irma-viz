use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{
    config::{
        cli::{CLIConfig, HeuristicsCLI, IOArgsCLI, PlotSpecificCLI, PlotToggleCLI},
        matrices::MatrixTypes,
        toml::{HeuristicsPlots, PercentVizOptionTOML, PlotSpecificTOML, TOMLConfig},
    },
    diagnostics::{
        PlotError,
        Severity::{self, Failure},
    },
    warn,
};

/// A parsed IO Config
#[derive(Debug)]
pub struct IOConfig {
    /// Path to the destination directory for created plots
    pub output_path: PathBuf,
    /// Format of the output files
    pub output_format: OutputFormat,
    /// Path to tables directory for irma run
    pub table_path: PathBuf,
    /// Path to matrix_path for irma run
    pub matrix_path: PathBuf,
}

/// takes the input root for an IRMA run and returns paths for the `tables/` and `matrices/` directories
pub fn get_directory_paths(input_root: &Path) -> (PathBuf, PathBuf) {
    (input_root.join("tables"), input_root.join("matrices"))
}

impl IOArgsCLI {
    /// Parses IO args by setting `output_path` to `input_root/figures` if no
    /// `output_path` is otherwise specified
    pub fn parse_io_args(self, output_format: OutputFormat) -> IOConfig {
        let IOArgsCLI {
            input_root,
            output_path,
        } = self;
        let output_path = output_path
            .clone()
            .unwrap_or_else(|| input_root.join("figures"));
        let (table_path, matrix_path) = get_directory_paths(&input_root);
        IOConfig {
            output_path,
            output_format,
            table_path,
            matrix_path,
        }
    }
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PercentVizOption {
    Sankey,
    // holds a bool for true/false based on whether the input is paired
    Pie(bool),
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
#[derive(Debug, Clone, Copy)]
pub struct ReadPercentConfig {
    pub viz_option: PercentVizOption,
}

/// Parsed and merged plot-specific options
#[derive(Debug)]
pub struct PlotSpecificConfig {
    pub coverage: CoverageConfig,
    pub read_percent: ReadPercentConfig,
    pub cluster_config: ClusterConfig,
    pub heuristic: HeuristicsConfig,
}

/// Disables heuristics subplots based on the result of their threshold
/// validation
fn disable_if_invalid(
    validation: Result<(), PlotError>,
    plots_enabled: bool,
    enabled_plots: &mut HeuristicsPlots,
    disable_plots: impl FnOnce(&mut HeuristicsPlots),
    warning_suffix: &str,
) {
    if !plots_enabled {
        return;
    }

    if let Err(err) = validation {
        warn(Severity::Warning, format!("{err}; {warning_suffix}"));
        disable_plots(enabled_plots);
    }
}

/// Validates the heuristics thresholds and disables the corresponding
/// heuristics subplots for invalid values.
fn validate_or_disable_heuristics(
    heuristics: HeuristicsCLI,
    toggles: &mut PlotToggles,
    enabled_plots: &mut HeuristicsPlots,
) {
    if !toggles.heuristics {
        // don't need to bother validating, no heuristics plot being created
        return;
    }

    // --min-variant-frequency disables allele_frequency and frequency_subplot
    disable_if_invalid(
        validate_finite_range(
            "--min-variant-frequency",
            heuristics.min_variant_frequency,
            0.0,
            1.0,
        ),
        enabled_plots.allele_frequency || enabled_plots.frequency_subplot,
        enabled_plots,
        |enabled_plots| {
            enabled_plots.allele_frequency = false;
            enabled_plots.frequency_subplot = false;
        },
        "skipping heuristics frequency plots",
    );

    // --min-confidence-not-sequencer-error disables confidence_hist
    disable_if_invalid(
        validate_finite_range(
            "--min-confidence-not-sequencer-error",
            heuristics.min_confidence_not_sequencer_error,
            0.0,
            1.0,
        ),
        enabled_plots.confidence_hist,
        enabled_plots,
        |enabled_plots| enabled_plots.confidence_hist = false,
        "skipping heuristics confidence histogram",
    );

    // --min-variant-average-quality disables allele_quality and quality_subplot
    disable_if_invalid(
        validate_finite_range(
            "min-variant-average-quality",
            heuristics.min_variant_average_quality,
            0.0,
            64.0,
        ),
        enabled_plots.allele_quality || enabled_plots.quality_subplot,
        enabled_plots,
        |enabled_plots| {
            enabled_plots.allele_quality = false;
            enabled_plots.quality_subplot = false;
        },
        "skipping heuristics quality plots",
    );

    // --min-variant-depth disables coverage_depth_hist
    let depth_validation = if !heuristics.min_variant_depth.is_finite()
        || heuristics.min_variant_depth < 1.0
    {
        Err(PlotError::ConfigError(format!(
            "Error: Value --min-variant-depth must be finite and greater than or equal to 1, {} was provided",
            heuristics.min_variant_depth
        )))
    } else {
        Ok(())
    };
    disable_if_invalid(
        depth_validation,
        enabled_plots.coverage_depth_hist,
        enabled_plots,
        |enabled_plots| enabled_plots.coverage_depth_hist = false,
        "skipping heuristics coverage histogram",
    );

    if !enabled_plots.check_any_enabled() {
        warn(
            Severity::Warning,
            PlotError::ConfigError(
                "Error: Heuristics plot enabled but no heuristics subplots were enabled; skipping heuristics plotting"
                    .to_string(),
            ),
        );
        toggles.heuristics = false;
    }
}

fn validate_finite_range(name: &str, value: f64, min: f64, max: f64) -> Result<(), PlotError> {
    if !value.is_finite() || value < min || value > max {
        return Err(PlotError::ConfigError(format!(
            "Value {name} must be finite and between {min} and {max}, {value} was provided"
        )));
    }
    Ok(())
}

impl PlotSpecificConfig {
    /// Merges the plot-specific arguments from the TOML and CLI and validates
    /// them.
    fn merge_plot_specifics(
        toml: PlotSpecificTOML,
        cli: PlotSpecificCLI,
        toggles: &mut PlotToggles,
    ) -> Self {
        // coverage options are only provided via TOML
        let coverage = toml.coverage;

        // heuristics options are only provided via CLI
        let mut enabled_plots = toml.heuristics.enabled_plots;
        // disables heuristics subplots whose relevant thresholds are invalid
        validate_or_disable_heuristics(cli.heuristics_args, toggles, &mut enabled_plots);
        let HeuristicsCLI {
            min_variant_average_quality: min_aq,
            min_variant_frequency: min_f,
            min_variant_depth: min_tcc,
            min_confidence_not_sequencer_error: min_conf,
        } = cli.heuristics_args;
        let heuristic = HeuristicsConfig {
            min_aq,
            min_f,
            min_tcc,
            min_conf,
            enabled_plots,
        };

        // gets read percentage viz option. if 'Pie' is enabled, but `--paired`
        // argument is not provided in CLI, READ_PERCENTAGES will be skipped and
        // this warning will be logged
        let viz_option = match toml.read_percent.viz_option {
            PercentVizOptionTOML::Sankey => PercentVizOption::Sankey,
            PercentVizOptionTOML::Pie => {
                let paired = match cli.paired {
                    Some(paired) => paired,
                    None => {
                        if toggles.read_percentages {
                            warn(Failure,
                                "READ_PERCENTAGES plot enabled with 'Pie' selected for viz_option, but `--paired` CLI argument not provided. Skipping READ_PERCENTAGES".to_string(),
                            );
                            toggles.read_percentages = false;
                        }
                        false
                    }
                };
                PercentVizOption::Pie(paired)
            }
        };
        let read_percent = ReadPercentConfig { viz_option };

        let cluster_config = ClusterConfig {
            // cluster option is provided via TOML
            cluster_option: toml.cluster_config.cluster_option,
            // matrix types are provided via TOML
            matrix_types: toml.cluster_config.matrix_types,
            // tree height comes from TOML + CLI
            tree_height: merge_toml_cli_value(toml.cluster_config.tree_height, cli.tree_height),
        };

        // we only validate `tree_height` if clustermap is enabled and cluster
        // option is Tree. if tree_height is invalid, we warn and skip
        // clustermap plots
        if toggles.clustermap && cluster_config.cluster_option == ClusterOption::Tree {
            match validate_finite_range("tree_height", cluster_config.tree_height, 0.0, 1.0) {
                Ok(_) => {}
                Err(err) => {
                    warn(Failure, format!("{err}, skipping Clustermap plots"));
                    toggles.clustermap = false;
                }
            }
        }

        PlotSpecificConfig {
            coverage,
            read_percent,
            cluster_config,
            heuristic,
        }
    }
}

#[derive(Debug)]
pub struct ParsedConfig {
    pub plot_toggles: PlotToggles,
    pub io_args: IOConfig,
    pub plot_specific: PlotSpecificConfig,
}

impl ParsedConfig {
    /// Merges all configuration options from the TOML and CLI into a
    /// [`ParsedConfig`] and performs validation on those options.
    pub fn merge_configs(toml: TOMLConfig, cli: CLIConfig) -> ParsedConfig {
        // takes the io args from the cli and pairs them with the output format
        // from the TOML
        let io_args = cli.io_args.parse_io_args(toml.output_options.output_format);

        let mut plot_toggles =
            PlotToggles::merge_plot_toggles(toml.plot_toggles, cli.enabled_plots);

        let plot_specific = PlotSpecificConfig::merge_plot_specifics(
            toml.plot_specific,
            cli.plot_specific_args,
            &mut plot_toggles,
        );

        ParsedConfig {
            plot_toggles,
            io_args,
            plot_specific,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "output_format")]
#[serde(rename_all = "snake_case")]
pub enum OutputFormatRaw {
    Pdf,
    Svg,
}

impl<'de> Deserialize<'de> for OutputFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match OutputFormatRaw::deserialize(deserializer)? {
            OutputFormatRaw::Svg => OutputFormat::Svg,
            OutputFormatRaw::Pdf if cfg!(feature = "pdf") => OutputFormat::Pdf,
            OutputFormatRaw::Pdf => {
                warn(
                    Severity::Warning,
                    "User specified 'output_format = \"pdf\"' but IRMA-viz not compiled with the PDF feature, switching to SVG.",
                );
                OutputFormat::Svg
            }
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
    Pdf,
    Svg,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Pdf => write!(f, ".pdf"),
            OutputFormat::Svg => write!(f, ".svg"),
        }
    }
}

/// Threshold values for heuristics plots to be passed via CLI
#[derive(Debug, Copy, Clone)]
pub struct HeuristicsConfig {
    /// Minimum average allele quality score heuristic for calling insertion &
    /// single nucleotide variants
    pub min_aq: f64,
    /// Minimum frequency heuristic for calling single nucleotide variants
    pub min_f: f64,
    /// Minimum coverage depth heuristic (total coverage count) for calling
    /// variants
    pub min_tcc: f64,
    /// Minimum confidence not machine error for single nucleotide variants
    pub min_conf: f64,
    pub enabled_plots: HeuristicsPlots,
}
