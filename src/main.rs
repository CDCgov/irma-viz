use crate::{
    config::{
        CLIConfig, ClusterOption, ClusterTargets, ConfigMergeSummary, MatrixType, ParsedConfig,
        PercentVizOption, discover_clustermap_targets, discover_coverage_targets,
        discover_heuristics_targets, load_config,
    },
    data::{AllAlleles, AllVariants, Coverage, PairingStats, ReadCounts, SankeyVec, SquareMatrix},
    diagnostics::{
        PlotError,
        Severity::{self},
        print_results, warn,
    },
    plots::{
        clustermap::{plot_clustermap, plot_heat_phylo},
        coverage::plot_coverage,
        heuristics::plot_heuristics,
        read_percentages::{plot_perc_pies, plot_perc_sankey},
    },
};
use clap::Parser;
use std::{fs, process::ExitCode};

mod config;
mod data;
#[cfg(feature = "demo")]
mod demo;
mod diagnostics;
mod plots;

const EXIT_PARTIAL_FAILURE: u8 = 3;

#[derive(Debug, Default)]
struct PlotRunSummary {
    rendered: Vec<String>,
    had_failures: bool,
}

/// Warns that a specific plot (with optional target) is being skipped, because
/// of the provided error. Wraps the warning in the [`warn`] function to provide
/// time stamp.
///
/// Always uses the [`Severity::Warning`] type, since this is
/// severity level we would expect for a single plot within a plot type failing
/// to be created
fn warn_plot_error(plot_type: &str, target: Option<&str>, err: &PlotError) {
    match target {
        Some(target) => warn(
            Severity::Warning,
            format!("skipping {plot_type} plot for '{target}': {err}"),
        ),
        None => warn(
            Severity::Warning,
            format!("skipping {plot_type} plot: {err}"),
        ),
    }
}

/// Run the program
fn main() -> ExitCode {
    let mut exit_code = ExitCode::SUCCESS;

    let cli = CLIConfig::parse();
    let config_path = cli.config.clone();
    #[cfg(feature = "demo")]
    let demo_target = cli.demo_target.clone();

    let toml = match load_config(&config_path).map_err(|err| {
        PlotError::InvalidData(format!("loading config from '{config_path}': {err}"))
    }) {
        Ok(toml) => toml,
        Err(err) => {
            warn(Severity::Failure, err);
            return ExitCode::FAILURE;
        }
    };

    let ConfigMergeSummary {
        cfg,
        had_config_failures,
    } = ParsedConfig::merge_configs(toml, cli);

    if had_config_failures {
        exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
    }

    #[cfg(feature = "demo")]
    if let Some(target) = demo_target.as_deref() {
        let mut cfg = cfg;
        cfg.io_args.output_path = std::path::PathBuf::from("demo");

        if let Err(err) = ensure_output_dir(&cfg)
            .map_err(|err| PlotError::IOError("creating demo output directory".to_string(), err))
        {
            warn(Severity::Failure, err);
            return ExitCode::FAILURE;
        }

        return match demo::run_demo(&mut cfg, target)
            .map_err(|err| PlotError::RenderError(err.to_string()))
        {
            Ok(()) => {
                warn(
                    Severity::Success,
                    format!("rendered demo bundle for target '{target}'"),
                );
                ExitCode::SUCCESS
            }
            Err(err) => {
                warn(Severity::Failure, err);
                ExitCode::FAILURE
            }
        };
    }

    if let Err(err) = ensure_output_dir(&cfg)
        .map_err(|err| PlotError::IOError("creating output directory".to_string(), err))
    {
        warn(Severity::Failure, err);
        return ExitCode::FAILURE;
    }

    // runs read_percentages and handles possible io or rendering errors that
    // could arise
    if cfg.plot_toggles.read_percentages
        && let Err(err) = run_read_percentages(&cfg)
    {
        warn_plot_error("read_percentages", None, &err);
        exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
    }

    if cfg.plot_toggles.heuristics {
        match run_heuristics(&cfg) {
            Ok(summary) => {
                let had_partial_failure = summary.had_failures || summary.rendered.is_empty();
                print_results(summary.rendered, "heuristics");
                if had_partial_failure {
                    exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
                }
            }
            Err(err) => {
                warn(Severity::Failure, err);
                exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
            }
        }
    }

    if cfg.plot_toggles.coverage {
        match run_coverage(&cfg) {
            Ok(summary) => {
                let had_partial_failure = summary.had_failures || summary.rendered.is_empty();
                print_results(summary.rendered, "coverage");
                if had_partial_failure {
                    exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
                }
            }

            Err(err) => {
                warn(Severity::Failure, err);
                exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
            }
        }
    }

    if cfg.plot_toggles.clustermap {
        match run_clustermap(&cfg) {
            Ok(summary) => {
                let had_partial_failure = summary.had_failures || summary.rendered.is_empty();
                print_results(summary.rendered, "clustermap");
                if had_partial_failure {
                    exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
                }
            }
            Err(err) => {
                warn(Severity::Failure, err);
                exit_code = ExitCode::from(EXIT_PARTIAL_FAILURE);
            }
        }
    }
    exit_code
}

/// Checks if the output directory exists, otherwise creates it.
fn ensure_output_dir(cfg: &ParsedConfig) -> Result<(), std::io::Error> {
    let output_dir = cfg.io_args.output_path.as_path();
    if !output_dir.as_os_str().is_empty() {
        fs::create_dir_all(output_dir)?;
    }

    Ok(())
}

/// Finds the `READ_COUNTS` file, then creates either a sankey or pie plot and
/// renders it via [`plot_perc_pies`]
///
/// ### Errors
///
/// Passes up up either an IO error from reading or parsing the `READ_COUNTS`
/// file, or a rendering error or IO error from [`plot_perc_sankey`]
fn run_read_percentages(cfg: &ParsedConfig) -> Result<(), PlotError> {
    let read_counts_path = cfg.io_args.table_path.join("READ_COUNTS.txt");
    match cfg.plot_specific.read_percent.viz_option {
        PercentVizOption::Sankey => {
            let sankey_vec = SankeyVec::import_from_file(&read_counts_path).map_err(|err| {
                PlotError::IOError(
                    format!(
                        "failed to read `READ_COUNTS` from '{}'",
                        read_counts_path.display()
                    ),
                    err,
                )
            })?;

            plot_perc_sankey(sankey_vec, cfg)
        }
        PercentVizOption::Pie(_) => {
            let read_counts = ReadCounts::import_from_file(&read_counts_path).map_err(|err| {
                PlotError::IOError(
                    format!(
                        "failed to  read `READ_COUNTS` from '{}'",
                        read_counts_path.display()
                    ),
                    err,
                )
            })?;

            plot_perc_pies(read_counts, cfg)
        }
    }
}

/// Discovers all possible heuristics targets, then attempts to plot each one,
/// using [`run_heuristics_for_target`]. Errors that arise during plot creation
/// are handled and reported within the loop. Returns a [`PlotRunSummary`] of
/// rendered targets and whether any targets failed.
///
/// ## Errors
///
/// Will return an error if IO operations within [`discover_heuristics_targets`]
/// fail.
fn run_heuristics(cfg: &ParsedConfig) -> Result<PlotRunSummary, PlotError> {
    let targets = discover_heuristics_targets(cfg)?;
    let mut summary = PlotRunSummary::default();
    for target in targets {
        let res = run_heuristics_for_target(cfg, &target);
        match res {
            Ok(_) => summary.rendered.push(target),
            Err(err) => {
                summary.had_failures = true;
                warn_plot_error("heuristics", Some(&target), &err);
            }
        }
    }
    Ok(summary)
}

/// Imports the data and creates the plot for a heuristics multiplot for a
/// single target.
///
/// ## Errors
///
/// Passes up an error if there is an error parsing [`AllAlleles`], or if an IO
/// Error arises during [`plot_heuristics`]
fn run_heuristics_for_target(cfg: &ParsedConfig, target: &str) -> Result<(), PlotError> {
    let all_alleles_path = cfg
        .io_args
        .table_path
        .join(format!("{target}-allAlleles.txt"));
    let allele_data = AllAlleles::import_from_file(&all_alleles_path).map_err(|err| {
        PlotError::MissingData(format!(
            "failed to read all-alleles data from '{}': {err}",
            all_alleles_path.display()
        ))
    })?;

    plot_heuristics(allele_data, cfg, target)
}

/// Discovers all possible coverage targets, then attempts to plot each one,
/// using [`run_coverage_for_target`]. Errors that arise during plot creation
/// are handled and reported within the loop. Returns a [`PlotRunSummary`] of
/// rendered targets and whether any targets failed.
///
/// ## Errors
///
/// Will return an error if IO operations within [`discover_coverage_targets`]
/// fail.
fn run_coverage(cfg: &ParsedConfig) -> Result<PlotRunSummary, PlotError> {
    let coverage_targets = discover_coverage_targets(cfg)?;
    let mut summary = PlotRunSummary::default();
    for target in coverage_targets {
        match run_coverage_for_target(cfg, &target) {
            Ok(_) => summary.rendered.push(target),
            Err(err) => {
                summary.had_failures = true;
                warn_plot_error("coverage", Some(&target), &err);
            }
        }
    }
    Ok(summary)
}

/// Imports the data and creates the coverage plot for a single target.
///
/// ## Errors
///
/// Passes up an error if there is an error parsing [`Coverage`], if there is an
/// error parsing [`AllVariants`], if there is an error parsing [`PairingStats`],
/// or if an IO Error arises during [`plot_coverage`]
fn run_coverage_for_target(cfg: &ParsedConfig, target: &str) -> Result<(), PlotError> {
    let coverage_path = cfg
        .io_args
        .table_path
        .join(format!("{target}-coverage.txt"));
    let coverage = Coverage::import_from_file(&coverage_path).map_err(|err| {
        PlotError::MissingData(format!(
            "failed to read coverage data from '{}': {err}",
            coverage_path.display()
        ))
    })?;

    let variants_path = cfg
        .io_args
        .table_path
        .join(format!("{target}-variants.txt"));
    let variants = AllVariants::import_from_file(&variants_path).map_err(|err| {
        PlotError::IOError(
            format!(
                "failed to read variants data from '{}'",
                variants_path.display()
            ),
            err,
        )
    })?;

    let pairing_stats_path = cfg
        .io_args
        .table_path
        .join(format!("{target}-pairingStats.txt"));
    let pairing_stats = PairingStats::import_from_file(&pairing_stats_path).map_err(|err| {
        PlotError::IOError(
            format!(
                "failed to read pairing-stats data from '{}'",
                pairing_stats_path.display()
            ),
            err,
        )
    })?;

    plot_coverage(coverage, variants, pairing_stats, cfg, target)
}

/// Discovers all possible clustermap targets, then attempts to plot each one,
/// using [`run_clustermap_for_target`]. Errors that arise during plot creation
/// are handled and reported within the loop. Returns a [`PlotRunSummary`] of
/// rendered target-matrix pairs and whether any targets failed.
///
/// ## Errors
///
/// Will return an error if IO operations within
/// [`discover_clustermap_targets`] fail.
fn run_clustermap(cfg: &ParsedConfig) -> Result<PlotRunSummary, PlotError> {
    let cluster_targets = discover_clustermap_targets(cfg)?;
    let mut summary = PlotRunSummary::default();

    for target in cluster_targets.variant_targets() {
        match run_clustermap_for_target(cfg, &cluster_targets, &target) {
            Ok(rendered_matrix_types) if !rendered_matrix_types.is_empty() => {
                summary.rendered.extend(rendered_matrix_types)
            }
            // skips if it's empty
            Ok(_) => {}
            Err(err) => {
                summary.had_failures = true;
                warn_plot_error("clustermap", Some(&target), &err);
            }
        }
    }

    Ok(summary)
}

/// Imports the variants data and creates clustermap plots for each enabled
/// matrix type available for a single target.
///
/// ## Errors
///
/// Passes up an error if there is an error parsing [`AllVariants`], if fewer
/// than two variants are present for the target, or if IO errors arise while
/// preparing clustermap inputs. Errors from individual matrix-type renders are
/// handled and reported within the loop.
fn run_clustermap_for_target(
    cfg: &ParsedConfig,
    cluster_targets: &ClusterTargets,
    target: &str,
) -> Result<Vec<String>, PlotError> {
    let clustermap_targets = cfg
        .plot_specific
        .cluster_config
        .matrix_types
        .enabled_matrix_types()
        .into_iter()
        .filter(|matrix_type| cluster_targets.targets_for(*matrix_type).contains(target))
        .collect::<Vec<_>>();

    let variants_path = cfg
        .io_args
        .table_path
        .join(format!("{target}-variants.txt"));
    let variants = AllVariants::import_from_file(&variants_path).map_err(|err| {
        PlotError::IOError(
            format!(
                "failed to read variants data from '{}'",
                variants_path.display()
            ),
            err,
        )
    })?;

    // this shouldn't ever trigger because if there is not enough variants then
    // the target will not get added as a valid target
    if variants.positions.len() <= 1 {
        return Err(PlotError::MissingData(format!(
            "no clustermap data found for target '{target}'"
        )));
    }

    let mut rendered_matrix_types = Vec::new();

    for matrix_type in clustermap_targets {
        match run_clustermap_for_matrix_type(cfg, target, matrix_type) {
            Ok(()) => {
                rendered_matrix_types.push(format!("{target}-{}", matrix_type.display_name()))
            }
            Err(err) => {
                let plot_type = match cfg.plot_specific.cluster_config.cluster_option {
                    ClusterOption::Clustermap => {
                        format!("clustermap {}", matrix_type.display_name())
                    }
                    ClusterOption::Tree => format!("tree {}", matrix_type.display_name()),
                };
                warn_plot_error(&plot_type, Some(target), &err);
            }
        }
    }

    Ok(rendered_matrix_types)
}

/// Imports the matrix data and creates a clustermap or tree plot for a single
/// target and matrix type.
///
/// ## Errors
///
/// Passes up an error if there is an error parsing [`SquareMatrix`], or if an
/// IO error arises during [`plot_clustermap`] or [`plot_heat_phylo`].
fn run_clustermap_for_matrix_type(
    cfg: &ParsedConfig,
    target: &str,
    matrix_type: MatrixType,
) -> Result<(), PlotError> {
    let sqm_path = cfg
        .io_args
        .matrix_path
        .join(format!("{target}{}", matrix_type.file_suffix()));
    let sqm = SquareMatrix::import_from_file(&sqm_path).map_err(|err| {
        PlotError::InvalidData(format!(
            "reading square matrix data from '{}': {err}",
            sqm_path.display()
        ))
    })?;

    match cfg.plot_specific.cluster_config.cluster_option {
        ClusterOption::Clustermap => plot_clustermap(sqm, cfg, target, matrix_type.display_name()),
        ClusterOption::Tree => plot_heat_phylo(sqm, cfg, target, matrix_type.display_name()),
    }
}
