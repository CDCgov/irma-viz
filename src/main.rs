//! TODO: Docs

use crate::{
    config::{PlottingArgs, apply_cli_overrides},
    data::{AllAlleles, Coverage, PairingStats, SankeyVec, SquareMatrix, Variants},
    plots::{
        clustermap::kuva_clustermap, coverage::kuva_coverage, heuristics::kuva_density,
        kuva_histogram, load_config, render_plots, sankey::kuva_sankey,
    },
};
use anyhow::{Context, Result};
use clap::Parser;
use kuva::prelude::{Layout, Plot};
use std::fs;
mod config;
mod data;
mod plots;

// taken from IRMA_RES/scripts/heuristicDiagram.R
const NUM_BINS: usize = 50;

fn main() -> Result<()> {
    let cli = PlottingArgs::parse();

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

    let mut plots: Vec<(String, (Vec<Plot>, Layout))> = Vec::new();

    // sankey
    if let Some(read_counts_path) = cfg.plots.sankey_path {
        let sankey_vec = SankeyVec::import_from_file(&read_counts_path).with_context(|| {
            format!(
                "Failed to import Read Counts data from: \'{}\'",
                &read_counts_path.display()
            )
        })?;
        let sankey = kuva_sankey(sankey_vec);
        plots.push((String::from("sankey.svg"), sankey));
    }

    // heuristics
    if let Some(all_alleles_path) = cfg.plots.heuristics_path {
        let allele_data = AllAlleles::import_from_file(&all_alleles_path).with_context(|| {
            format!(
                "Failed to import All Alleles data from \'{}\'",
                &all_alleles_path.display()
            )
        })?;

        let avg_qualities = allele_data
            .average_qualities
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let density = kuva_density(
            avg_qualities.clone(),
            cfg.constants.min_aq,
            "Density of average allele quality",
        );
        plots.push((String::from("allele_quality_density.svg"), density));

        // quality density subplot
        if let Some(min_aq) = cfg.constants.min_aq {
            let mut limited_density = kuva_density(
                avg_qualities.clone(),
                None,
                &format!("Density of average allele quality to {}", min_aq),
            );
            limited_density.1.x_axis_max = Some(min_aq);
            plots.push((
                format!("allele_quality_density_to_{}.svg", min_aq),
                limited_density,
            ));
        }

        if cfg.plots.density_observed {
            let observed_densities = allele_data.frequencies.clone();
            let observed_frequency = kuva_density(
                observed_densities,
                cfg.constants.min_f,
                "Density of observed frequency",
            );
            plots.push((
                String::from("observed_frequency_density.svg"),
                observed_frequency,
            ));
        }

        if cfg.plots.observed_8
            && let Some(min_f) = cfg.constants.min_f
        {
            let mut limited_frequency = kuva_density(
                allele_data.frequencies.clone(),
                None,
                &format!("Density of observed frequency (to {}%)", min_f),
            );
            limited_frequency.1.x_axis_max = Some(min_f);
            plots.push((
                format!("observed_frequency_density_to_{}.svg", min_f),
                limited_frequency,
            ));
        }

        if cfg.plots.coverage {
            let coverage_depths = allele_data.totals;
            let coverage_histogram =
                kuva_histogram(coverage_depths, NUM_BINS, None, "Histogram of coverage");
            plots.push((String::from("coverage_histogram.svg"), coverage_histogram));
        }

        if cfg.plots.confidence {
            let confidences = allele_data
                .confidence_not_mac_errs
                .into_iter()
                .flatten()
                .collect();
            let confidence_histogram = kuva_histogram(
                confidences,
                NUM_BINS,
                cfg.constants.min_conf,
                "Histogram of confidence not machine error, non-zero",
            );
            plots.push((
                String::from("confidence_histogram.svg"),
                confidence_histogram,
            ));
        }
    }

    // heatmap and variant phase clustering tree
    if let Some(sqm_path) = cfg.plots.sqm_path {
        let sqm = SquareMatrix::import_from_file(&sqm_path).with_context(|| {
            format!(
                "Failed to import Square Matrix data from \'{}\'",
                &sqm_path.display()
            )
        })?;

        let clustermap = kuva_clustermap(sqm.clone());
        plots.push((String::from("EXPENRD.svg"), clustermap));
    }

    if let (Some(coverage_path), Some(variants_path), Some(pairing_stats_path)) = (
        cfg.plots.coverage_path,
        cfg.plots.variants_path,
        cfg.plots.pairing_stats_path,
    ) {
        let coverage_data = Coverage::import_from_file(&coverage_path).with_context(|| {
            format!(
                "Failed to import Coverage data from \'{}\'",
                &coverage_path.display()
            )
        })?;
        let variants = Variants::import_from_file(&variants_path).with_context(|| {
            format!(
                "Failed to import Variants data from \'{}\'",
                &variants_path.display()
            )
        })?;
        let pairing_stats =
            PairingStats::import_from_file(&pairing_stats_path).with_context(|| {
                format!(
                    "Failed to import Pairing Stats data from \'{}\'",
                    &pairing_stats_path.display()
                )
            })?;

        let coverage_plot = kuva_coverage(coverage_data, variants, pairing_stats);
        plots.push((String::from("coverage.svg"), coverage_plot))
    }

    render_plots(plots, &cfg.output.path)?;

    Ok(())
}
