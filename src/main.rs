use anyhow::{Context, Result};
use clap::Parser;
use plotters::{coord::Shift, prelude::*};
use std::fs;

mod config;
use config::{Config, PlotsConfig};

use crate::config::{PlottingArgs, apply_cli_overrides};

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

    let cfg = load_config(&cli.config)
        .with_context(|| format!("Loading config from {}", cli.config))?;

    let cfg = apply_cli_overrides(cfg, &cli);

    let enabled = enabled_plots(&cfg.plots);
    if enabled.is_empty() {
        anyhow::bail!("No plots enabled in config.toml ([plots] section).");
    }

    if let Some(parent) = std::path::Path::new(&cfg.output.path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Creating output dir {:?}", parent))?;
        }
    }

    render_svg_grid(&cfg.output.path, cfg.output.width, cfg.output.height, &enabled)?;

    println!(
        "Wrote {} plot(s) to {}",
        enabled.len(),
        cfg.output.path
    );
    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("Reading {path}"))?;
    let cfg: Config = toml::from_str(&s).with_context(|| format!("Parsing {path}"))?;
    Ok(cfg)
}

fn enabled_plots(p: &PlotsConfig) -> Vec<PlotKind> {
    let mut v = Vec::new();
    if p.density_average {
        v.push(PlotKind::DensityAverage);
    }
    if p.density_8 {
        v.push(PlotKind::Density8);
    }
    if p.density_observed {
        v.push(PlotKind::DensityObserved);
    }
    if p.observed_8 {
        v.push(PlotKind::Observed8);
    }
    if p.coverage {
        v.push(PlotKind::CoverageHist);
    }
    if p.confidence {
        v.push(PlotKind::ConfidenceHist);
    }
    v
}


/// if making all six plots: 3x2 grid. otherwise, we make a compact layout
/// https://stackoverflow.com/a/2480939
fn grid_dims(n: usize, height: u32, width: u32) -> (usize, usize) {
    let aspect_ratio = width as f32/height as f32;

    if n == 6 {
        return (3, 2);
    }

    let cols = (n as f32 * aspect_ratio).sqrt().ceil() as usize;
    let rows = (n as f32 / aspect_ratio).sqrt().ceil() as usize;
    (rows.max(1), cols.max(1))
}

fn render_svg_grid(path: &str, width: u32, height: u32, plots: &[PlotKind]) -> Result<()> {
    let root = SVGBackend::new(path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let (rows, cols) = grid_dims(plots.len(), height, width);
    let areas = root.split_evenly((rows, cols));

    for (i, kind) in plots.iter().enumerate() {
        if i >= areas.len() {
            break;
        }
        draw_one(&areas[i], *kind)
            .with_context(|| format!("Drawing plot {:?} in cell {}", kind, i))?;
    }

    root.present()?;
    Ok(())
}

fn draw_one(area: &DrawingArea<SVGBackend, Shift>, kind: PlotKind) -> Result<()> {
    area.fill(&WHITE)?;

    match kind {
        PlotKind::DensityAverage => draw_line_plot(area, "density_average", line_data_1())?,
        PlotKind::Density8 => draw_line_plot(area, "density_8", line_data_2())?,
        PlotKind::DensityObserved => draw_line_plot(area, "density_observed", line_data_3())?,
        PlotKind::Observed8 => draw_line_plot(area, "observed_8", line_data_4())?,
        PlotKind::CoverageHist => draw_hist(area, "coverage", hist_data_1())?,
        PlotKind::ConfidenceHist => draw_hist(area, "confidence", hist_data_2())?,
    }
    Ok(())
}

// these are placeholders for the real plots

fn draw_line_plot(
    area: &DrawingArea<SVGBackend, Shift>,
    title: &str,
    data: Vec<(f64, f64)>,
) -> Result<()> {
    let (xmin, xmax) = data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (x, _)| {
            (mn.min(*x), mx.max(*x))
        });
    let (ymin, ymax) = data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (_, y)| {
            (mn.min(*y), mx.max(*y))
        });

    let mut chart = ChartBuilder::on(area)
        .margin(10)
        .caption(title, ("sans-serif", 18))
        .x_label_area_size(28)
        .y_label_area_size(38)
        .build_cartesian_2d(xmin..xmax, ymin..ymax)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("x")
        .y_desc("y")
        .draw()?;

    chart.draw_series(LineSeries::new(data, &BLACK))?;
    Ok(())
}

fn draw_hist(area: &DrawingArea<SVGBackend, Shift>, title: &str, samples: Vec<f64>) -> Result<()> {
    let bins = 20usize;
    let (minv, maxv) = samples.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), v| {
        (mn.min(*v), mx.max(*v))
    });
    let width = (maxv - minv).max(1e-12);
    let bin_w = width / bins as f64;

    let mut counts = vec![0u32; bins];
    for v in samples {
        let mut idx = ((v - minv) / bin_w) as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx as usize >= bins {
            idx = (bins - 1) as isize;
        }
        counts[idx as usize] += 1;
    }
    let max_count = *counts.iter().max().unwrap_or(&0) as i32;

    let mut chart = ChartBuilder::on(area)
        .margin(10)
        .caption(title, ("sans-serif", 18))
        .x_label_area_size(28)
        .y_label_area_size(38)
        .build_cartesian_2d(minv..maxv, 0..max_count)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("value")
        .y_desc("count")
        .draw()?;

    chart.draw_series(counts.iter().enumerate().map(|(i, &c)| {
        let x0 = minv + i as f64 * bin_w;
        let x1 = x0 + bin_w;
        Rectangle::new([(x0, 0), (x1, c as i32)], BLACK.filled())
    }))?;

    Ok(())
}

// placeholders for the data, this is where we'll read in the actual info from the matrices/intermediate files

fn line_data_1() -> Vec<(f64, f64)> {
    (0..200).map(|i| (i as f64 / 10.0, (i as f64 / 42.0).sin())).collect()
}
fn line_data_2() -> Vec<(f64, f64)> {
    (0..200).map(|i| (i as f64 / 10.0, (i as f64 / 70.0).sin())).collect()
}
fn line_data_3() -> Vec<(f64, f64)> {
    (0..200).map(|i| (i as f64 / 10.0, (i as f64 / 32.0).sin())).collect()
}
fn line_data_4() -> Vec<(f64, f64)> {
    (0..200).map(|i| (i as f64 / 10.0, (i as f64 / 69.0).sin())).collect()
}
fn hist_data_1() -> Vec<f64> {
    (0..2000).map(|i| ((i as f64 * 0.01).sin() + 1.0) * 0.5).collect()
}
fn hist_data_2() -> Vec<f64> {
    (0..2000).map(|i| ((i as f64 * 0.02).sin() + 1.0) * 0.5).collect()
}
