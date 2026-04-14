//! TODO: Docs

use crate::{
    config::{Args, apply_cli_overrides, load_config},
    data::{AllAlleles, SankeyVec, SquareMatrix},
    plots::{clustermap::plot_clustermap, plot_heuristics, sankey::plot_sankey},
};
use anyhow::{Context, Result};
use clap::Parser;
use std::{fs, path::PathBuf};
mod config;
mod data;
mod plots;

// taken from IRMA_RES/scripts/heuristicDiagram.R
// const NUM_BINS: usize = 50;

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
        let read_counts_path = cfg.input.data_path.join("READ_COUNTS.txt");
        let sankey_vec = SankeyVec::import_from_file(&read_counts_path).with_context(|| {
            format!(
                "Failed to import Read Counts data from: \'{}\'",
                &read_counts_path.display()
            )
        })?;

        plot_sankey(sankey_vec, &cfg)?
    }

    for target in &cfg.targets.list {
        // clustermap
        if cfg.plot_toggles.clustermap {
            let sqm_path = cfg.input.matrix_path.join(format!("{target}-EXPENRD.sqm"));
            let sqm = SquareMatrix::import_from_file(&sqm_path).with_context(|| {
                format!(
                    "Failed to import Square Matrix data from \'{}\'",
                    &sqm_path.display()
                )
            })?;

            plot_clustermap(sqm, &cfg, target)?
        }

        // heursitics multi-plot
        if cfg.plot_toggles.heuristics {
            // input paths
            let all_alleles_path = cfg.input.data_path.join(format!("{target}-allAlleles.txt"));
            // heuristics
            let allele_data =
                AllAlleles::import_from_file(&all_alleles_path).with_context(|| {
                    format!(
                        "Failed to import All Alleles data from \'{}\'",
                        &all_alleles_path.display()
                    )
                })?;

            plot_heuristics(&allele_data, &cfg)?
        }

        // let density = kuva_density(allele_data, cfg);
        // plots.push((String::from("allele_quality_density.svg"), density));

        // // quality density subplot
        // if let Some(min_aq) = cfg.constants.min_aq {
        //     let mut limited_density = kuva_density(
        //         avg_qualities.clone(),
        //         None,
        //         &format!("Density of average allele quality to {}", min_aq),
        //     );
        //     limited_density.1.x_axis_max = Some(min_aq);
        //     plots.push((
        //         format!("allele_quality_density_to_{}.svg", min_aq),
        //         limited_density,
        //     ));
        // }

        // if cfg.plots.density_observed {
        //     let observed_densities = allele_data.frequencies.clone();
        //     let observed_frequency = kuva_density(
        //         observed_densities,
        //         cfg.constants.min_f,
        //         "Density of observed frequency",
        //     );
        //     plots.push((
        //         String::from("observed_frequency_density.svg"),
        //         observed_frequency,
        //     ));
        // }

        // if cfg.plots.observed_8
        //     && let Some(min_f) = cfg.constants.min_f
        // {
        //     let mut limited_frequency = kuva_density(
        //         allele_data.frequencies.clone(),
        //         None,
        //         &format!("Density of observed frequency (to {}%)", min_f),
        //     );
        //     limited_frequency.1.x_axis_max = Some(min_f);
        //     plots.push((
        //         format!("observed_frequency_density_to_{}.svg", min_f),
        //         limited_frequency,
        //     ));
        // }

        // if cfg.plots.coverage {
        //     let coverage_depths = allele_data.totals;
        //     let coverage_histogram =
        //         kuva_histogram(coverage_depths, NUM_BINS, None, "Histogram of coverage");
        //     plots.push((String::from("coverage_histogram.svg"), coverage_histogram));
        // }

        // if cfg.plots.confidence {
        //     let confidences = allele_data
        //         .confidence_not_mac_errs
        //         .into_iter()
        //         .flatten()
        //         .collect();
        //     let confidence_histogram = kuva_histogram(
        //         confidences,
        //         NUM_BINS,
        //         cfg.constants.min_conf,
        //         "Histogram of confidence not machine error, non-zero",
        //     );
        //     plots.push((
        //         String::from("confidence_histogram.svg"),
        //         confidence_histogram,
        //     ));
        // }
    }

    // if let (Some(coverage_path), Some(variants_path), Some(pairing_stats_path)) = (
    //     cfg.input.coverage_path,
    //     cfg.input.variants_path,
    //     cfg.input.pairing_stats_path,
    // ) {
    //     let coverage_data = Coverage::import_from_file(&coverage_path).with_context(|| {
    //         format!(
    //             "Failed to import Coverage data from \'{}\'",
    //             &coverage_path.display()
    //         )
    //     })?;
    //     let variants = Variants::import_from_file(&variants_path).with_context(|| {
    //         format!(
    //             "Failed to import Variants data from \'{}\'",
    //             &variants_path.display()
    //         )
    //     })?;
    //     let pairing_stats =
    //         PairingStats::import_from_file(&pairing_stats_path).with_context(|| {
    //             format!(
    //                 "Failed to import Pairing Stats data from \'{}\'",
    //                 &pairing_stats_path.display()
    //             )
    //         })?;

    //     let coverage_plot = kuva_coverage(coverage_data, variants, pairing_stats);
    //     plots.push((String::from("coverage.svg"), coverage_plot))
    // }

    // render_plots(plots, &cfg.output.path)?;

    Ok(())
}
