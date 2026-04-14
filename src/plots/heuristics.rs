use crate::{config::Config, data::AllAlleles};
use anyhow::Result;
use kuva::{plot::Histogram, prelude::*};

pub fn kuva_density(all_alleles: AllAlleles) -> DensityPlot {
    let data = all_alleles
        .average_qualities
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    DensityPlot::new().with_data(data)
}

#[allow(unused)]
pub fn kuva_histogram(
    data: Vec<f64>,
    num_bins: usize,
    reference_line: Option<f64>,
    title: &str,
) -> (Vec<Plot>, Layout) {
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let hist = Histogram::new()
        .with_data(data)
        .with_bins(num_bins)
        .with_range((min, max))
        .with_color("steelblue");

    let plots = vec![Plot::Histogram(hist)];
    let mut layout = Layout::auto_from_plots(&plots)
        .with_title(title)
        .with_x_label("Value")
        .with_y_label("Count");

    if let Some(line) = reference_line {
        layout = layout.with_reference_line(ReferenceLine::vertical(line))
    }
    (plots, layout)
}

pub fn plot_heuristics(all_allele_data: &AllAlleles, cfg: &Config) -> Result<()> {
    Ok(())
}
