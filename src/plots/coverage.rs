use crate::{
    config::{Config, CoverageColorOption},
    data::{AllVariants, Coverage, PairingStats},
    plots::{render_multiplot, render_plot},
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
fn map_allele_color(frequency: f64, freq_range: (f64, f64)) -> String {
    let colormap = ColorMap::Viridis;
    let (min, max) = freq_range;

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
    variants: AllVariants,
    pairing_stats: PairingStats,
    cfg: &Config,
    target: &str,
) -> Result<()> {
    let min_y = min_coverage(&coverage);

    let coverage_plot = kuva_coverage(coverage);

    let mut coverage_layout = Layout::auto_from_plots(&coverage_plot)
        .with_y_axis_min(min_y - min_y * 0.05)
        .with_y_label("Coverage depth")
        .with_x_label(format!("{target} position"))
        .with_show_grid(false);

    for i in 0..variants.positions.len() {
        coverage_layout = coverage_layout.with_reference_line(
            ReferenceLine::vertical(variants.positions[i] as f64)
                // color is chosen based on the plot color option specified in
                // the config.
                .with_color(match cfg.plot_specific.coverage.color_option {
                    crate::config::CoverageColorOption::Nucleotide => {
                        get_allele_color(variants.minority_alleles[i])
                    }
                    crate::config::CoverageColorOption::Frequency => {
                        let use_allele_color = pairing_stats
                            .data
                            .get("ExpectedErrorRate")
                            .is_none_or(|ee| variants.minority_frequencies.data[i] >= *ee);

                        if use_allele_color {
                            map_allele_color(
                                variants.minority_frequencies.data[i],
                                (
                                    variants.minority_frequencies.min,
                                    variants.minority_frequencies.max,
                                ),
                            )
                        } else {
                            "#000000".to_owned()
                        }
                    }
                })
                .with_label(variants.minority_alleles[i])
                .with_dasharray("8 0"),
        );
    }

    let filename = format!("{target}-coverageDiagram.svg");

    // skip bar making and multiplot if using frequency for coloring, or if no
    // variants
    if cfg.plot_specific.coverage.color_option == CoverageColorOption::Nucleotide
        && !variants.minority_frequencies.data.is_empty()
    {
        let (coverage_bar, bar_layout) = coverage_bar(&variants, pairing_stats);

        let scene = Figure::new(2, 1)
            .with_plots(vec![coverage_plot, coverage_bar])
            .with_layouts(vec![coverage_layout, bar_layout])
            .render();

        render_multiplot(&scene, cfg.output.path.clone(), &filename)
    } else {
        render_plot(
            (&filename, (coverage_plot, coverage_layout)),
            cfg.output.path.clone(),
        )
    }
}

/// Creates a bar of the minor variants, using labels such as A2C, for a
/// concensus A with a variant C. The bars are colored based on the nucleotide
/// of the variant, with heights based on the observed frequency of that variant.
pub fn coverage_bar(variants: &AllVariants, pairing_stats: PairingStats) -> (Vec<Plot>, Layout) {
    let mut bar = BarPlot::new();
    let expected = pairing_stats.data.get("ExpectedErrorRate");

    if let Some(value) = expected {
        bar = bar.with_colored_bar("Expected Error", *value, "black");
    }

    for i in 0..variants.positions.len() {
        let label = format!(
            "{}2{} ({})",
            variants.consensus_alleles[i], variants.minority_alleles[i], variants.positions[i]
        );

        bar = bar.with_colored_bar(
            label,
            variants.minority_frequencies.data[i],
            get_allele_color(variants.minority_alleles[i]),
        );
    }

    let bar = vec![bar.into()];

    let bar_layout = Layout::auto_from_plots(&bar)
        .with_x_tick_rotate(70.0)
        .with_x_label("Minor Variants")
        .with_y_label("Observed Frequency");

    if let Some(value) = expected {
        let bar_layout =
            bar_layout.with_reference_line(ReferenceLine::horizontal(*value).with_color("black"));
        (bar, bar_layout)
    } else {
        (bar, bar_layout)
    }
}
