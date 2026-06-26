//! TODO: Docs

use crate::{
    config::{
        Args, ClusterOption, PercentVizOption, apply_cli_overrides, get_directory_paths,
        load_config, resolve_targets,
    },
    data::{AllAlleles, AllVariants, Coverage, PairingStats, ReadCounts, SankeyVec, SquareMatrix},
    plots::{
        clustermap::{plot_clustermap, plot_heat_phylo},
        coverage::plot_coverage,
        heuristics::plot_heuristics,
        read_percentages::{plot_perc_pies, plot_perc_sankey},
    },
};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
mod config;
mod data;
mod plots;

fn main() -> Result<()> {
    let cli = Args::parse();

    let cfg =
        load_config(&cli.config).with_context(|| format!("Loading config from {}", cli.config))?;

    let cfg = apply_cli_overrides(cfg, &cli);

    let plot_targets = resolve_targets(&cfg)?;
    plot_targets.check_missing_targets(&cfg.plot_toggles);

    let io_args = cfg.io_args()?;
    let (table_path, matrix_path) = get_directory_paths(&io_args.input_root);
    let output_path = cfg.output_path()?;

    // create output directory
    if let output_dir = std::path::Path::new(&output_path)
        && !output_dir.as_os_str().is_empty()
    {
        fs::create_dir_all(output_dir)?;
    }

    // Read Counts
    if cfg.plot_toggles.read_percentages {
        let read_counts_path = table_path.join("READ_COUNTS.txt");

        if !read_counts_path.is_file() {
            eprintln!(
                "Warning: read percentages plotting was enabled but required file '{}' was not found",
                read_counts_path.display()
            );
        } else {
            match cfg.plot_specific.read_percent.viz_option {
                PercentVizOption::Sankey => {
                    let sankey_vec =
                        SankeyVec::import_from_file(&read_counts_path).with_context(|| {
                            format!(
                                "Failed to import Read Counts data from: \'{}\'",
                                read_counts_path.display()
                            )
                        })?;

                    plot_perc_sankey(sankey_vec, &cfg)
                        .with_context(|| "Error plotting READ_PERCENTAGES.svg")?
                }
                PercentVizOption::Pie => {
                    let read_counts = ReadCounts::import_from_file(&read_counts_path)
                        .with_context(|| {
                            format!(
                                "Failed to import Read Counts data from: \'{}\'",
                                read_counts_path.display()
                            )
                        })?;

                    plot_perc_pies(read_counts, &cfg)
                        .with_context(|| "Error plotting READ_PERCENTAGES.svg")?
                }
            }
        }
    }

    for target in &plot_targets.heuristics {
        let all_alleles_path = table_path.join(format!("{target}-allAlleles.txt"));
        let allele_data = AllAlleles::import_from_file(&all_alleles_path).with_context(|| {
            format!(
                "Failed to import All Alleles data from \'{}\'",
                all_alleles_path.display()
            )
        })?;

        plot_heuristics(allele_data, &cfg, target)
            .with_context(|| format!("Error plotting {target}-heuristics.svg"))?
    }

    for target in plot_targets.variant_targets() {
        let variants_path = table_path.join(format!("{target}-variants.txt"));
        let variants = AllVariants::import_from_file(&variants_path).with_context(|| {
            format!(
                "Failed to import Variants data from \'{}\'",
                variants_path.display()
            )
        })?;

        if plot_targets.clustermap.contains(&target) && variants.positions.len() > 1 {
            let sqm_path = matrix_path.join(format!("{target}-EXPENRD.sqm"));
            let sqm = SquareMatrix::import_from_file(&sqm_path).with_context(|| {
                format!(
                    "Failed to import Square Matrix data from \'{}\'",
                    sqm_path.display()
                )
            })?;
            match cfg.plot_specific.cluster_config.cluster_option {
                ClusterOption::Clustermap => plot_clustermap(sqm, &cfg, &target)
                    .with_context(|| format!("Error plotting {target}-EXPENRD.svg"))?,
                ClusterOption::Tree => plot_heat_phylo(sqm, &cfg, &target)
                    .with_context(|| format!("Error plotting {target}-EXPENRD.svg"))?,
            }
        }

        if plot_targets.coverage.contains(&target) {
            let coverage_path = table_path.join(format!("{target}-coverage.txt"));
            let coverage = Coverage::import_from_file(&coverage_path).with_context(|| {
                format!(
                    "Failed to import Coverage data from \'{}\'",
                    coverage_path.display()
                )
            })?;

            let pairing_stats_path = table_path.join(format!("{target}-pairingStats.txt"));
            let pairing_stats =
                PairingStats::import_from_file(&pairing_stats_path).with_context(|| {
                    format!(
                        "Failed to import Pairing Stats data from \'{}\'",
                        pairing_stats_path.display()
                    )
                })?;

            plot_coverage(coverage, variants, pairing_stats, &cfg, &target)
                .with_context(|| format!("Error plotting {target}-coverageDiagram.svg"))?
        }
    }

    Ok(())
}
