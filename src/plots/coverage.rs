use crate::{
    config::Config,
    data::{Coverage, PairingStats, Variant, Variants},
    plots::render_multiplot,
};
use anyhow::Result;
use kuva::prelude::*;

/// For coloring allele reference lines based on the variant nucleotide
fn get_allele_color(allele: char) -> String {
    let color = match allele {
        'A' => "#1F77B4",
        'C' => "#FF7F0E",
        'G' => "#2CA02C",
        'T' => "#D62728",
        _ => "#FFFFFF",
    };
    color.to_owned()
}

/// For coloring allele reference lines based on
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
        .map(
            |Variant {
                 position: _,
                 consensus_allele: _,
                 minority_allele: _,
                 minority_frequency,
             }| *minority_frequency,
        )
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

    //for (position, _consensus_allele, minority_allele, minority_frequency) in variants.data {
    for variant in &variants.data {
        coverage_layout = coverage_layout.with_reference_line(
            ReferenceLine::vertical(variant.position as f64)
                // color is chosen based on the plot color option specified in
                // the config.
                .with_color(match cfg.plot_specific.coverage.color_option {
                    crate::config::CoverageColorOption::Nucleotide => {
                        get_allele_color(variant.minority_allele)
                    }
                    crate::config::CoverageColorOption::Frequency => {
                        let use_allele_color = pairing_stats
                            .data
                            .get("ExpectedErrorRate")
                            .is_none_or(|ee| variant.minority_frequency >= *ee);

                        if use_allele_color {
                            map_allele_color(variant.minority_frequency, freq_range)
                        } else {
                            "#000000".to_owned()
                        }
                    }
                })
                .with_label(variant.minority_allele)
                .with_dasharray("8 0"),
        );
    }

    let coverage_bar = coverage_bar(&variants, pairing_stats);

    // TODO: Add bar chart to multiplot here
    let scene = Figure::new(2, 1)
        .with_plots(vec![coverage_plot, coverage_bar])
        .with_layouts(vec![coverage_layout, bar_layout])
        .render();

    let filename = format!("{target}-coverageDiagram.svg");
    render_multiplot(&scene, cfg.output.path.clone(), filename.as_str())
}

/// Creates a bar of the minor variants, using labels such as A2C, for a
/// concensus A with a variant C. The bars are colored based on the nucleotide
/// of the variant, with heights based on the observed frequency of that variant.
pub fn coverage_bar(variants: &Variants, pairing_stats: PairingStats) -> BarPlot {
    let mut bar = BarPlot::new();

    for variant in &variants.data {
        let label = format!(
            "{}2{} ({})",
            variant.consensus_allele, variant.minority_allele, variant.position
        );

        bar = bar.with_colored_bar(
            label,
            variant.minority_frequency,
            get_allele_color(variant.minority_allele),
        );
    }

    bar
}
