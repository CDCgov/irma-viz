//! TODO: Docs

use crate::{
    config::{Args, apply_cli_overrides, load_config},
    data::{AllAlleles, Coverage, PairingStats, SankeyVec, SquareMatrix, Variants},
    plots::{
        clustermap::plot_clustermap, coverage::plot_coverage, heuristics::plot_heuristics,
        read_percentages::plot_sankey,
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

    // create output directory
    if let output_dir = std::path::Path::new(&cfg.output.path)
        && !output_dir.as_os_str().is_empty()
    {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Creating output dir {}", output_dir.display()))?;
    }

    // Read Counts
    if cfg.plot_toggles.read_percentages {
        let read_counts_path = cfg.input.table_path.join("READ_COUNTS.txt");
        let sankey_vec = SankeyVec::import_from_file(&read_counts_path).with_context(|| {
            format!(
                "Failed to import Read Counts data from: \'{}\'",
                &read_counts_path.display()
            )
        })?;

        plot_sankey(sankey_vec, &cfg).with_context(|| "Error plotting READ_PERCENTAGES.svg")?
    }

    for target in &cfg.targets.list {
        // heuristics multi-plot
        if cfg.plot_toggles.heuristics {
            let all_alleles_path = cfg
                .input
                .table_path
                .join(format!("{target}-allAlleles.txt"));
            let allele_data =
                AllAlleles::import_from_file(&all_alleles_path).with_context(|| {
                    format!(
                        "Failed to import All Alleles data from \'{}\'",
                        &all_alleles_path.display()
                    )
                })?;

            plot_heuristics(allele_data, &cfg, target)
                .with_context(|| format!("Error plotting {target}-heuristics.svg"))?
        }

        if cfg.plot_toggles.clustermap || cfg.plot_toggles.coverage {
            let variants_path = cfg.input.table_path.join(format!("{target}-variants.txt"));
            let variants = Variants::import_from_file(&variants_path).with_context(|| {
                format!(
                    "Failed to import Variants data from \'{}\'",
                    &variants_path.display()
                )
            })?;

            // clustermap
            if cfg.plot_toggles.clustermap && variants.data.len() > 1 {
                let sqm_path = cfg.input.matrix_path.join(format!("{target}-EXPENRD.sqm"));
                let sqm = SquareMatrix::import_from_file(&sqm_path).with_context(|| {
                    format!(
                        "Failed to import Square Matrix data from \'{}\'",
                        &sqm_path.display()
                    )
                })?;

                plot_clustermap(sqm, &cfg, target)
                    .with_context(|| format!("Error plotting {target}-EXPENRD.svg"))?
            }

            if cfg.plot_toggles.coverage {
                let coverage_path = cfg.input.table_path.join(format!("{target}-coverage.txt"));
                let coverage = Coverage::import_from_file(&coverage_path).with_context(|| {
                    format!(
                        "Failed to import Coverage data from \'{}\'",
                        &coverage_path.display()
                    )
                })?;

                let pairing_stats_path = cfg
                    .input
                    .table_path
                    .join(format!("{target}-pairingStats.txt"));
                let pairing_stats = PairingStats::import_from_file(&pairing_stats_path)
                    .with_context(|| {
                        format!(
                            "Failed to import Pairing Stats data from \'{}\'",
                            &pairing_stats_path.display()
                        )
                    })?;

                plot_coverage(coverage, variants, pairing_stats, &cfg, target)
                    .with_context(|| format!("Error plotting {target}-coverageDiagram.svg"))?
            }
        }
    }

    Ok(())
}
