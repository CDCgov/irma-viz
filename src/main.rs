//! TODO: Docs

use crate::{
    config::{PlottingArgs, apply_cli_overrides},
    data::{AllAlleles, Coverage, PairingStats, SankeyVec, SquareMatrix, Variants},
    plots::{
        clustermap::kuva_clustermap, heuristics::kuva_density, kuva_histogram, load_config,
        render_plots, sankey::kuva_sankey
    },
};
use anyhow::{Context, Result};
use clap::Parser;
use kuva::prelude::{Layout, Plot};
use std::fs;
mod config;
mod data;
mod plots;

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
        let density = kuva_density(avg_qualities.clone());
        plots.push((String::from("density.svg"), density));

        // quality density subplot
        if let Some(min_aq) = cfg.constants.min_aq {
            let mut limited_density = kuva_density(avg_qualities.clone());
            limited_density.1.x_axis_max = Some(min_aq);
            plots.push((format!("density_to_{}.svg", min_aq), limited_density));
        }

        if cfg.plots.density_observed {
            let observed_densities = allele_data.frequencies.clone();
            let observed_frequency = kuva_density(observed_densities);
            plots.push((String::from("frequency.svg"), observed_frequency));
        }

        if let Some(min_f) = cfg.constants.min_f {
            let mut limited_frequency = kuva_density(allele_data.frequencies.clone());
            limited_frequency.1.x_axis_max = Some(min_f);
            plots.push((format!("frequency_to_{}.svg", min_f), limited_frequency));
        }

        if cfg.plots.coverage {
            let coverage_depths = allele_data.totals;
            let coverage_histogram = kuva_histogram(coverage_depths, 50);
            plots.push((String::from("coverage_histogram.svg"), coverage_histogram));
        }

        if cfg.plots.confidence {
            let confidences = allele_data
                .confidence_not_mac_errs
                .into_iter()
                .flatten()
                .collect();
            let confidence_histogram = kuva_histogram(confidences, 50);
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

    render_plots(plots, &cfg.output.path)?;

    // Coverage API demo
    if let Some(coverage_path) = cfg.plots.coverage_path {
        let coverage_data = Coverage::import_from_file(&coverage_path).with_context(|| {
            format!(
                "Failed to import Coverage data from \'{}\'",
                &coverage_path.display()
            )
        })?;

        let _coverages = coverage_data.coverages;
    }

    // Pairing Stats API demo
    if let Some(pairing_stats_path) = cfg.plots.pairing_stats_path {
        let pairing_stats =
            PairingStats::import_from_file(&pairing_stats_path).with_context(|| {
                format!(
                    "Failed to import Pairing Stats data from \'{}\'",
                    &pairing_stats_path.display()
                )
            })?;

        let _ps_data_example = pairing_stats.data.get("Observations");
    }

    // Variants API demo
    if let Some(variants_path) = cfg.plots.variants_path {
        let variants = Variants::import_from_file(&variants_path).with_context(|| {
            format!(
                "Failed to import Variants data from \'{}\'",
                &variants_path.display()
            )
        })?;

        if let Some((_con_allele_ex, _min_allele_ex)) = variants.data.get(&0) {
            // println!("Some variant exists!")
        } else {
            // println!("No variants exist!")
        }
    }

    Ok(())
}
