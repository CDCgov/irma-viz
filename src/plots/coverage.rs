//! Coverage-depth figures with minority-variant annotations.

use crate::{
    config::{CoverageColorOption, ParsedConfig},
    data::{AllVariants, Coverage, PairingStats},
    diagnostics::PlotError,
    plots::{render_multiplot, render_plot},
};
use kuva::{prelude::*, render::annotations::TextAnnotation};

/// Returns the corresponding color for a minority allele, based on nucleotide
/// identity, used in bars and reference lines.
///
/// Unsupported or unrecognized nucleotides are rendered white.
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

/// Maps a minority-allele frequency to a clamped Viridis color.
///
/// A degenerate frequency range maps to the palette midpoint
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

/// Builds a filled coverage-depth line plot from [`Coverage`] observations.
pub fn kuva_coverage(coverage: Coverage) -> Vec<Plot> {
    vec![
        LinePlot::new()
            .with_data(coverage.position.into_iter().zip(coverage.coverage))
            .with_fill()
            .with_stroke_width(0.3)
            .into(),
    ]
}

/// Creates a coverage plot for a target and renders it to a file. If enabled
/// and if variants exist, a multiplot will instead be created with a bar plot
/// showing the frequencies of variants
///
/// ## Errors
///
/// Passes up any IO Error that might arise from [`render_multiplot`] or
/// [`render_plot`]
pub fn plot_coverage(
    coverage: Coverage,
    variants: AllVariants,
    pairing_stats: PairingStats,
    cfg: &ParsedConfig,
    target: &str,
) -> Result<(), PlotError> {
    const OFFSET: f64 = 20.5;

    let coverage_plot = kuva_coverage(coverage.clone());
    let expected_error = pairing_stats.data.get("ExpectedErrorRate").copied();
    let show_bar = cfg.plot_specific.coverage.color_option == CoverageColorOption::Nucleotide
        && !variants.minority_frequencies.data.is_empty();
    let freq_range = (
        variants.minority_frequencies.min,
        variants.minority_frequencies.max,
    );

    let mut coverage_layout = Layout::auto_from_plots(&coverage_plot)
        .with_clamp_axis()
        .with_y_label("Coverage depth")
        .with_x_label(format!("{target} position"))
        .with_show_grid(false);
    let mut coverage_bar_plot = BarPlot::new();

    if let Some(value) = expected_error
        && show_bar
    {
        coverage_bar_plot =
            coverage_bar_plot.with_colored_bar(format!("exp. err. = {value:.2E}"), value, "black");
    }

    for (index, (((&position, &consensus_allele), &minority_allele), &minority_frequency)) in
        variants
            .positions
            .iter()
            .zip(&variants.consensus_alleles)
            .zip(&variants.minority_alleles)
            .zip(&variants.minority_frequencies.data)
            .enumerate()
    {
        coverage_layout = coverage_layout.with_reference_line(
            ReferenceLine::vertical(position as f64)
                .with_color(
                    if expected_error.is_none_or(|ee| minority_frequency >= ee) {
                        match cfg.plot_specific.coverage.color_option {
                            crate::config::CoverageColorOption::Nucleotide => {
                                get_allele_color(minority_allele)
                            }
                            crate::config::CoverageColorOption::Frequency => {
                                map_allele_color(minority_frequency, freq_range)
                            }
                        }
                    } else {
                        "#000000".to_owned()
                    },
                )
                .with_dasharray("8 0"),
        );

        if position <= coverage.coverage.len() && position != 0 {
            let (min_y, _) = coverage_layout.y_range;
            // Check if this label would overlap with any previous labels
            // TODO: Try to break this by going off the bottom axis; test
            let mut annotation_y_pos =
                (coverage.coverage[position.saturating_sub(1)] + min_y) / 2.0;
            for prev_index in (0..index).rev() {
                let prev_pos = variants.positions[prev_index];
                let distance = position.abs_diff(prev_pos);
                if OFFSET > distance as f64 && annotation_y_pos - 2.5 * OFFSET > min_y {
                    annotation_y_pos -= 2.5 * OFFSET;
                }
            }
            coverage_layout = coverage_layout.with_annotation(TextAnnotation::new(
                minority_allele,
                (position as f64) + OFFSET,
                annotation_y_pos,
            ));
        }

        if show_bar {
            let label = format!("{consensus_allele}2{minority_allele}");
            coverage_bar_plot = coverage_bar_plot.with_colored_bar(
                label,
                minority_frequency,
                get_allele_color(minority_allele),
            );
        }
    }

    let filename = format!("{target}-coverageDiagram");

    // skip bar making and multiplot if using frequency for coloring, or if no
    // variants
    if show_bar {
        let (coverage_bar, mut bar_layout) = coverage_bar(coverage_bar_plot, expected_error);
        for (idx, (position, min_freq)) in variants
            .positions
            .iter()
            .zip(&variants.minority_frequencies.data)
            .enumerate()
        {
            bar_layout = bar_layout.with_annotation(
                TextAnnotation::new(position.to_string(), (idx + 2) as f64, min_freq / 2.0)
                    .with_color("#ffffff"),
            );
        }
        bar_layout = bar_layout.with_tick_format(TickFormat::Fixed(3));

        let scene = Figure::new(2, 2)
            .with_structure(vec![vec![0, 1], vec![2, 3]])
            .with_plots(vec![coverage_plot, coverage_bar])
            .with_layouts(vec![coverage_layout, bar_layout])
            .render();

        render_multiplot(
            &scene,
            &cfg.io_args.output_path,
            &filename,
            cfg.io_args.output_format,
        )
    } else {
        render_plot(
            (&filename, (coverage_plot, coverage_layout)),
            &cfg.io_args.output_path,
            cfg.io_args.output_format,
        )
    }
}

/// Creates a bar of the minor variants, using labels such as A2C, for a
/// concensus A with a variant C. The bars are colored based on the nucleotide
/// of the variant, with heights based on the observed frequency of that variant.
///
/// /// When present, `expected` adds a horizontal expected-error reference line.
pub fn coverage_bar(bar: BarPlot, expected: Option<f64>) -> (Vec<Plot>, Layout) {
    let bar = vec![bar.into()];

    let bar_layout = Layout::auto_from_plots(&bar)
        .with_x_label("Minor Variants")
        .with_y_label("Observed Frequency");

    if let Some(value) = expected {
        let bar_layout =
            bar_layout.with_reference_line(ReferenceLine::horizontal(value).with_color("black"));
        (bar, bar_layout)
    } else {
        (bar, bar_layout)
    }
}
