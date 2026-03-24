use crate::{
    config::{PlottingArgs, apply_cli_overrides},
<<<<<<< HEAD
    data::{AllAllelesData, ReadCountsData},
    plots::{enabled_plots, load_config, make_plots, render_plots},
=======
    data::ReadCountsData,
    plots::{enabled_plots, load_config, make_plots, render_plots, sankey::{kuva_sankey, to_sankey_vec}},
>>>>>>> 28954ae (wip)
};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
mod config;
mod data;
mod plots;

/// stores kind of plot and the data that goes along with it
#[derive(Clone, Debug)]
enum PlotData<'a> {
    DensityAverage(Vec<(f64, f64)>),
    Density8(Vec<(f64, f64)>),
    DensityObserved(Vec<(f64, f64)>),
    Observed8(Vec<(f64, f64)>),
    CoverageHist(Vec<f64>),
    ConfidenceHist(Vec<f64>),
    ReadMapSankey(Vec<(&'a str, &'a str, f64)>),
}

fn main() -> Result<()> {
    let cli = PlottingArgs::parse();

    let cfg =
        load_config(&cli.config).with_context(|| format!("Loading config from {}", cli.config))?;

    let cfg = apply_cli_overrides(cfg, &cli);

    let enabled = enabled_plots(&cfg.plots);
    let num_plots = enabled.len();
    if num_plots == 0 {
        anyhow::bail!("No plots enabled in config.toml ([plots] section).");
    }

    // create output directory
    if let Some(parent) = std::path::Path::new(&cfg.output.path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("Creating output dir {:?}", parent))?;
    }

    let plots = make_plots(enabled);
    render_plots(plots, &cfg.output.path);

    println!("Wrote {} plot(s) to {}", num_plots, cfg.output.path);

    // Demos of data API
    let filename = "test_data/READ_COUNTS.txt";
    let read_counts_data = ReadCountsData::import_from_file(filename)
        .with_context(|| format!("Cannot import Read Counts data from: \'{}\'", filename))?;
    let _record_data = read_counts_data
        .record_data_map
        .get("4-A_NP")
        .expect("ya beefed it");
    println!("{record_data:#?}");

    let filename2 = "test_data/A_PA-allAlleles.txt";
    let all_alleles_data = AllAllelesData::import_from_file(filename2)
        .with_context(|| format!("Cannot import All Alleles data from: \'{}\'", filename2))?;
    let count_data = all_alleles_data.counts[0];
    println!("this count: {count_data}");


    //println!("{record_data:#?}");
    let transformed = to_sankey_vec(&read_counts_data);
    let sankey = kuva_sankey(transformed);
    let plots = vec![(String::from("Sankey.svg"), sankey)];
    render_plots(plots, &cfg.output.path);
    Ok(())
}
