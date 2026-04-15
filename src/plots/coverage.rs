use crate::{
    config::Config,
    data::{Coverage, PairingStats, Variants},
    plots::render_multiplot,
};
use anyhow::Result;
use kuva::prelude::*;

#[allow(unused)]
fn get_allele_color(allele: u8) -> String {
    let color = match allele {
        b'A' => "#1F77B4",
        b'C' => "#FF7F0E",
        b'G' => "#2CA02C",
        b'T' => "#D62728",
        _ => "#FFFFFF",
    };
    color.to_owned()
}

fn map_allele_color(frequency: f64, freq_range: Option<(f64, f64)>) -> String {
    let colormap = ColorMap::Viridis;
    let (min, max) = freq_range.expect("Only gets called if Variant data is not empty");

    let normalized_freq = if min == max {
        0.5
    } else {
        ((frequency - min) / (max - min)).clamp(0.0, 1.0)
    };

    colormap.map(normalized_freq)
}

fn min_coverage(coverage: &Coverage) -> f64 {
    coverage
        .coverage_by_position
        .iter()
        .map(|&(_, pos)| pos)
        .min_by(|a, b| a.total_cmp(b))
        .expect("Data is checked for empty and Nones at import.")
}

fn freq_range(variants: &Variants) -> Option<(f64, f64)> {
    variants
        .data
        .iter()
        .map(|(_, _, _, x)| *x)
        .fold(None, |acc, x| {
            Some(match acc {
                None => (x, x),
                Some((min, max)) => (
                    if x.total_cmp(&min).is_lt() { x } else { min },
                    if x.total_cmp(&max).is_gt() { x } else { max },
                ),
            })
        })
}

pub fn kuva_coverage(coverage: Coverage) -> Vec<Plot> {
    vec![
        LinePlot::new()
            .with_data(coverage.coverage_by_position)
            .with_fill()
            .with_stroke_width(0.3)
            .into(),
    ]
}

pub fn plot_coverage(
    coverage: Coverage,
    variants: Variants,
    pairing_stats: PairingStats,
    cfg: &Config,
    target: &str,
) -> Result<()> {
    let min_y = min_coverage(&coverage);
    let freq_range = freq_range(&variants);

    let coverage_plot = kuva_coverage(coverage);
    let mut coverage_layout = Layout::auto_from_plots(&coverage_plot)
        .with_y_axis_min(min_y - min_y * 0.05)
        .with_y_label("Coverage depth")
        .with_x_label(format!("{target} position"));

    for (position, _consensus_allele, minority_allele, minority_frequency) in variants.data {
        let (color, label) = match minority_allele {
            Some(minority_allele) => {
                let use_allele_color = pairing_stats
                    .data
                    .get("ExpectedErrorRate")
                    .is_none_or(|ee| minority_frequency >= *ee);

                let color = if use_allele_color {
                    map_allele_color(minority_frequency, freq_range)
                } else {
                    "#000000".to_owned()
                };

                (color, char::from(minority_allele).to_string())
            }
            None => (String::new(), String::new()),
        };

        coverage_layout = coverage_layout.with_reference_line(
            ReferenceLine::vertical(position)
                .with_color(color)
                .with_label(&label)
                .with_dasharray("8 0"),
        );
    }

    // TODO: Here is where bar chart creation goes

    // TODO: Add bar chart to multiplot here
    let scene = Figure::new(1, 1)
        .with_plots(vec![coverage_plot])
        .with_layouts(vec![coverage_layout])
        .render();

    let filename = format!("{target}-coverageDiagram.svg");
    render_multiplot(&scene, cfg.output.path.clone(), filename.as_str())
}
