use crate::config::Config;
use anyhow::{Context, Result};
use kuva::{plot::Histogram, prelude::*};
use std::fs;

pub fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("Reading {path}"))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("Parsing {path}"))?;
    Ok(cfg)
}

pub fn kuva_density(data: Vec<f64>) -> (Vec<Plot>, Layout) {
    let density = DensityPlot::new().with_data(data).with_color("steelblue");
    let plots: Vec<Plot> = vec![density.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density of average allele quality")
        .with_x_label("Allele quality")
        .with_y_label("Density");
    (plots, layout)
}

#[allow(unused)]
pub fn kuva_histogram(data: Vec<f64>, num_bins: usize) -> (Vec<Plot>, Layout) {
    // Compute range from data first.
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let hist = Histogram::new()
        .with_data(data)
        .with_bins(num_bins)
        .with_range((min, max))
        .with_color("steelblue");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram")
        .with_x_label("Value")
        .with_y_label("Count");

    (plots, layout)
}
