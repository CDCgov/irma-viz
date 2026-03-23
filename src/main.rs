use crate::{
    config::{PlottingArgs, apply_cli_overrides},
    data::ReadCountsData,
    plots::{enabled_plots, load_config, render_svg_grid},
};
use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
mod config;
mod data;
mod plots;

#[derive(Clone, Copy, Debug)]
enum PlotKind {
    DensityAverage,
    Density8,
    DensityObserved,
    Observed8,
    CoverageHist,
    ConfidenceHist,
}

fn main() -> Result<()> {
    let cli = PlottingArgs::parse();

    let cfg =
        load_config(&cli.config).with_context(|| format!("Loading config from {}", cli.config))?;

    let cfg = apply_cli_overrides(cfg, &cli);

    let enabled = enabled_plots(&cfg.plots);
    if enabled.is_empty() {
        anyhow::bail!("No plots enabled in config.toml ([plots] section).");
    }

    if let Some(parent) = std::path::Path::new(&cfg.output.path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("Creating output dir {:?}", parent))?;
    }

    render_svg_grid(
        &cfg.output.path,
        cfg.output.width,
        cfg.output.height,
        &enabled,
    )?;

    println!("Wrote {} plot(s) to {}", enabled.len(), cfg.output.path);

    let filename = "test_data/READ_COUNTS.txt";
    let read_counts_data = ReadCountsData::import_from_file(filename)
        .with_context(|| format!("Cannot import Read Counts data from: \'{}\'", filename))?;
    let record_data = read_counts_data
        .record_data_map
        .get("4-A_NP")
        .expect("ya beefed it");
    println!("{record_data:#?}");
    Ok(())
}
