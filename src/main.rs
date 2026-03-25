use crate::{
    config::{PlottingArgs, apply_cli_overrides},
    data::{AllAllelesData, ReadCountsData},
    plots::{load_config, render_plots, sankey::{kuva_sankey, to_sankey_vec}, heuristics::kuva_density},
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
    if let Some(parent) = std::path::Path::new(&cfg.output.path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("Creating output dir {:?}", parent))?;
    }

    let mut plots: Vec<(String, (Vec<Plot>, Layout))> = Vec::new();

    // sankey
    if let Some(read_counts_path) = cfg.plots.sankey {
        let read_counts_data = ReadCountsData::import_from_file(&read_counts_path)
        .with_context(|| format!("Cannot import Read Counts data from: \'{}\'", &read_counts_path.to_string_lossy()))?;
        let sankey = kuva_sankey(to_sankey_vec(&read_counts_data));
        plots.push((String::from("sankey.svg"), sankey));
    }

    // heuristics
    if let Some(all_alleles_path) = cfg.plots.heuristics_path {
        let allele_data = AllAllelesData::import_from_file(&all_alleles_path)?;
        
        let avg_qualities = allele_data.average_qualities.into_iter().flatten().collect::<Vec<_>>();
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
    }

    render_plots(plots, &cfg.output.path);
    Ok(())
}
