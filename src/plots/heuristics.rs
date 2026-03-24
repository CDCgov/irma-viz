use crate::{
    PlotData,
    config::{Config, PlotsConfig},
};
use kuva::{plot::{Histogram, line}, prelude::*};
use anyhow::{Context, Result};
use std::{fs};

pub fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("Reading {path}"))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("Parsing {path}"))?;
    Ok(cfg)
}

pub fn enabled_plots(p: &PlotsConfig) -> Vec<PlotData> {

    let line_data: Vec<(f64, f64)> = (0..=100)
        .map(|i| { let x = i as f64 * 0.1; (x, x.sin()) })
        .collect();

    let hist_data = line_data.clone().iter().map(|(f1, _f2)| *f1).collect::<Vec<_>>();

    let edges: Vec<(&str, &str, f64)> = vec![
    ("A", "B", 10.0),
    ("A", "C", 20.0),
    ("B", "D", 10.0),
    ("C", "D", 20.0),
    ];

    let mut v = Vec::new();
    if p.density_average {
        v.push(PlotData::DensityAverage(line_data.clone()));
    }
    if p.density_8 {
        v.push(PlotData::Density8(line_data.clone()));
    }
    if p.density_observed {
        v.push(PlotData::DensityObserved(line_data.clone()));
    }
    if p.observed_8 {
        v.push(PlotData::Observed8(line_data.clone()));
    }
    if p.coverage {
        v.push(PlotData::CoverageHist(hist_data.clone()));
    }
    if p.confidence {
        v.push(PlotData::ConfidenceHist(hist_data.clone()));
    }
    if let Some(path) = &p.sankey {
        v.push(PlotData::ReadMapSankey(edges.clone()));
    }
    v
}

pub fn kuva_line(data: Vec<(f64, f64)>) -> (Vec<Plot>, Layout) {

    let plot = LinePlot::new()
        .with_data(data)
        .with_color("steelblue");

    let plots: Vec<Plot> = vec![plot.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("My Plot")
        .with_x_label("X")
        .with_y_label("Y");
    (plots, layout)
}


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





