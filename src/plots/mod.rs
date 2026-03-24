pub mod heuristics;
pub mod sankey;

use std::path::{Path};

use kuva::{prelude::{Layout, Plot}, render_to_svg};

use crate::{PlotData, plots::sankey::kuva_sankey};

pub fn make_plot_layout(kind: PlotData) -> (String, (Vec<Plot>, Layout)) {
    match kind {
        PlotData::DensityAverage(data) => (String::from("density_average.svg"), kuva_line(data)),
        PlotData::Density8(data) => (String::from("density_8.svg"), kuva_line(data)),
        PlotData::DensityObserved(data) => (String::from("density_observed.svg"), kuva_line(data)),
        PlotData::Observed8(data) => (String::from("observed_8.svg"), kuva_line(data)),
        PlotData::CoverageHist(data) => (String::from("coverage_histogram.svg"), kuva_histogram(data, 20)),
        PlotData::ConfidenceHist(data) => (String::from("confidence_histogram.svg"), kuva_histogram(data, 20)),
        PlotData::ReadMapSankey(data) => (String::from("readmap_sankey.svg"), kuva_sankey(data))
    }
}

pub fn make_plots(plots: Vec<PlotData>) -> Vec<(String, (Vec<Plot>, Layout))> {
    let mut layouts = Vec::new();
    // placeholder data, will get this from inputs
    let data: Vec<(f64, f64)> = (0..=100)
        .map(|i| { let x = i as f64 * 0.1; (x, x.sin()) })
        .collect();

    for plot_kind in plots {
        layouts.push(make_plot_layout(plot_kind));
    }
    layouts
}

pub fn render_plots(plots: Vec<(String, (Vec<Plot>, Layout))>, outpath: impl AsRef<Path>) {
    for plot in plots {
        let (filename, (plots, layout)) = plot;
        let filepath = outpath.as_ref().join(filename);
        let svg = render_to_svg(plots, layout);
        std::fs::write(filepath, svg).unwrap();
    }
}


pub use heuristics::*;
