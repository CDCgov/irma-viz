//! Heuristic diagnostic figures for target-specific all-alleles data.

use crate::{
    config::{HeuristicsConfig, ParsedConfig},
    data::AllAlleles,
    diagnostics::{PlotError, Severity, warn},
    plots::render_multiplot,
    warn_plot_error,
};
use kuva::{plot::Histogram, prelude::*};

// target number of bins for histogram. Value from original IRMA R-script
const NUM_BINS: usize = 50;
// Number of sample points used to render each kernel-density curve.
const SAMPLES: usize = 1000;

/// Creates a heuristics multiplot for a target. Each of the six subplots can be
/// independently toggled off in the TOML; each also can individually fail if
/// there is no data, in which case they will be excluded from the output
///
/// ## Errors
///
/// Returns an error if no plots were able to be created, or passes up an IO
/// Error from [`render_multiplot`]
pub fn plot_heuristics(
    all_alleles: AllAlleles,
    cfg: &ParsedConfig,
    target: &str,
) -> Result<(), PlotError> {
    let HeuristicsConfig {
        min_aq,
        min_f,
        min_tcc,
        min_conf,
        enabled_plots,
    } = cfg.plot_specific.heuristic;

    let mut plots = Vec::new();
    let mut layouts = Vec::new();

    if enabled_plots.allele_quality {
        let average_qualities = &all_alleles.average_qualities;
        if average_qualities.data.is_empty() {
            warn_plot_error(
                "heuristics allele quality",
                Some(target),
                &PlotError::MissingData(String::from("no average quality data found")),
            );
        } else {
            let (aq_density, min_y, max_y) = kuva_dens(
                &average_qualities.data,
                average_qualities.min,
                average_qualities.max,
            );
            let aq_dens_layout = Layout::auto_from_plots(&aq_density)
                .with_title("Density of average allele quality")
                .with_x_axis_min(average_qualities.min)
                .with_x_axis_max(average_qualities.max)
                .with_y_axis_min(min_y - min_y * 0.05)
                .with_y_axis_max(max_y + max_y * 0.05)
                .with_reference_line(ReferenceLine::vertical(min_aq).with_dasharray("none"))
                .with_show_grid(false);

            plots.push(aq_density);
            layouts.push(aq_dens_layout);
        }
    }

    if enabled_plots.quality_subplot {
        let average_qualities = &all_alleles.average_qualities;
        if average_qualities.data.is_empty() {
            // don't need to warn a second time for the subplot if it's already
            // been warned once about the missing data
            if !enabled_plots.allele_quality {
                warn_plot_error(
                    "heuristics quality subplot",
                    Some(target),
                    &PlotError::MissingData(String::from("no average quality data found")),
                );
            }
        } else {
            let (limited_aq_density, min_y, max_y) =
                kuva_dens(&average_qualities.data, average_qualities.min, min_aq);
            let lim_aq_dens_layout = Layout::auto_from_plots(&limited_aq_density)
                .with_title(format!("to {min_aq}"))
                .with_x_axis_min(average_qualities.min)
                .with_x_axis_max(min_aq)
                .with_y_axis_min(min_y - min_y * 0.05)
                .with_y_axis_max(max_y + max_y * 0.05)
                .with_show_grid(false);

            plots.push(limited_aq_density);
            layouts.push(lim_aq_dens_layout);
        }
    }

    if enabled_plots.allele_frequency {
        let frequencies = &all_alleles.frequencies;
        if frequencies.is_empty() {
            warn_plot_error(
                "heuristics frequency",
                Some(target),
                &PlotError::MissingData(String::from("no allele frequency data found")),
            );
        } else {
            let (freq_density, min_y, max_y) = kuva_dens(frequencies, 0.0, 0.1);
            let freq_dens_layout = Layout::auto_from_plots(&freq_density)
                .with_title("Density of observed frequency (to 10%)")
                .with_x_axis_min(0.0)
                .with_x_axis_max(0.1)
                .with_y_axis_min(min_y - min_y * 0.05)
                .with_y_axis_max(max_y + max_y * 0.05)
                .with_reference_line(ReferenceLine::vertical(min_f).with_dasharray("none"))
                .with_show_grid(false);

            plots.push(freq_density);
            layouts.push(freq_dens_layout);
        }
    }

    if enabled_plots.frequency_subplot {
        let frequencies = &all_alleles.frequencies;
        if frequencies.is_empty() {
            if !enabled_plots.allele_frequency {
                warn_plot_error(
                    "heuristics frequency subplot",
                    Some(target),
                    &PlotError::MissingData(String::from("no allele frequency data found")),
                );
            }
        } else {
            let (lim_freq_dens, min_y, max_y) = kuva_dens(frequencies, 0.0, min_f);
            let lim_freq_dens_layout = Layout::auto_from_plots(&lim_freq_dens)
                .with_title(format!("to {min_f}"))
                .with_x_axis_min(0.0)
                .with_x_axis_max(min_f)
                .with_y_axis_min(min_y - min_y * 0.001)
                .with_y_axis_max(max_y + max_y * 0.001)
                .with_show_grid(false);

            plots.push(lim_freq_dens);
            layouts.push(lim_freq_dens_layout);
        }
    }

    if enabled_plots.coverage_depth_hist {
        if all_alleles.totals.data.is_empty() {
            warn_plot_error(
                "heuristics coverage depth histogram",
                Some(target),
                &PlotError::MissingData(String::from("no coverage depth data found")),
            );
        } else {
            match kuva_histogram(all_alleles.totals.data, NUM_BINS) {
                Ok(cov_hist) => {
                    let cov_hist_layout = Layout::auto_from_plots(&cov_hist)
                        .with_x_axis_min(0.0)
                        .with_x_axis_max(all_alleles.totals.upper_quantile + 1.0)
                        .with_reference_line(
                            ReferenceLine::vertical(min_tcc).with_dasharray("none"),
                        )
                        .with_show_grid(false)
                        .with_title("Histogram of coverage (Depth <= 20% Quantile)");

                    plots.push(cov_hist);
                    layouts.push(cov_hist_layout);
                }
                Err(err) => warn(
                    Severity::Warning,
                    format!("skipping coverage histogram in heuristics plot for '{target}': {err}"),
                ),
            }
        }
    }

    if enabled_plots.confidence_hist {
        let confidence_values = all_alleles.confidence_not_mac_errs;
        if confidence_values.is_empty() {
            warn_plot_error(
                "heuristics confidence not error histogram",
                Some(target),
                &PlotError::MissingData(String::from("no confidence data found")),
            );
        } else {
            match kuva_histogram(confidence_values, NUM_BINS) {
                Ok(confidence_histogram) => {
                    let confidence_hist_layout = Layout::auto_from_plots(&confidence_histogram)
                        .with_reference_line(
                            ReferenceLine::vertical(min_conf).with_dasharray("none"),
                        )
                        .with_show_grid(false)
                        .with_title("Histogram of confidence of not machine error, non-zero");

                    plots.push(confidence_histogram);
                    layouts.push(confidence_hist_layout);
                }
                Err(err) => warn(
                    Severity::Warning,
                    format!(
                        "skipping confidence histogram in heuristics plot for '{target}': {err}"
                    ),
                ),
            }
        }
    }

    if plots.is_empty() {
        return Err(PlotError::MissingData(format!(
            "could not create heuristics plot for '{target}'; no plottable panels were available"
        )));
    }

    let cols = if plots.len() > 1 { 2 } else { 1 };
    let rows = match plots.len() {
        1 | 2 => 1,
        3 | 4 => 2,
        5 | 6 => 3,
        _ => unreachable!(),
    };

    // Multi-Plot
    let scene = Figure::new(rows, cols)
        .with_plots(plots)
        .with_layouts(layouts)
        .render();

    let filename = format!("{target}-heuristics");
    render_multiplot(
        &scene,
        &cfg.io_args.output_path,
        filename.as_str(),
        cfg.io_args.output_format,
    )
}

/// Builds a Silverman-bandwidth kernel-density curve and y bounds over an x
/// interval.
fn kuva_dens(data: &[f64], x_lo: f64, x_hi: f64) -> (Vec<Plot>, f64, f64) {
    let bw = kuva::silverman_bandwidth(data);
    let n = data.len() as f64;
    let norm = 1.0 / (n * bw * (2.0 * std::f64::consts::PI).sqrt());

    let raw = { kuva::simple_kde(data, bw, SAMPLES) };

    let mut curve = Vec::with_capacity(raw.len());
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for (x, y) in raw {
        let y = y * norm;
        curve.push((x, y));
        if (x_lo..=x_hi).contains(&x) {
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    (vec![LinePlot::new().with_data(curve).into()], min_y, max_y)
}

/// Builds a histogram using explicit bin edges so the rendered plot follows
/// IRMA/R-style break choices instead of kuva's automatic binning.
fn kuva_histogram(data: Vec<f64>, num_bins: usize) -> Result<Vec<Plot>, PlotError> {
    if data.is_empty() {
        return Err(PlotError::MissingData(
            "histogram plot has no data".to_string(),
        ));
    }

    let breaks = pretty_breaks(&data, num_bins)?;
    let counts = histogram_counts(&data, &breaks);

    Ok(vec![
        Histogram::from_bins(breaks, counts)
            .with_color("#272727c2")
            .into(),
    ])
}

/// R-style pretty histogram breaks for a suggested number of bins.
///
/// This is inspired by the path used by `hist(x, breaks = n)` for ordinary
/// numeric vectors: use the data range, pick a 1/2/5/10 * 10^k cell width near
/// the requested width, then expand the endpoints to cell boundaries.
///
/// ## Errors
///
/// Can error if the minimum or maximum of the data is not a finite value
fn pretty_breaks(data: &[f64], suggested_bins: usize) -> Result<Vec<f64>, PlotError> {
    if suggested_bins == 0 {
        return Err(PlotError::ConfigError(
            "suggested histogram bin count must be greater than zero".to_string(),
        ));
    }

    let (min, max) = data
        .iter()
        .filter(|x| x.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &x| {
            (min.min(x), max.max(x))
        });

    if !min.is_finite() || !max.is_finite() {
        return Err(PlotError::MissingData(
            "histogram breaks require at least one finite value".to_string(),
        ));
    }

    // Degenerate ranges need padding so a constant-value histogram still has a
    // visible span; clamp probability-like values to their natural [0, 1] range.
    if min == max {
        if min == 0.0 {
            return Ok(vec![0.0, 1.0]);
        }

        let unit_interval = (0.0..=1.0).contains(&min);
        let min_width = if unit_interval { f64::EPSILON } else { 1.0 };
        let width = pretty_width((min.abs() * 0.2).max(min_width));

        let mut lower = min - width;
        let mut upper = min + width;

        if unit_interval {
            lower = lower.max(0.0);
            upper = upper.min(1.0);
        } else if min > 0.0 {
            lower = lower.max(0.0);
        } else {
            upper = upper.min(0.0);
        }

        if lower >= upper {
            lower = 0.0;
            upper = if unit_interval { 1.0 } else { min + width };
        }

        return Ok(vec![zero_if_tiny(lower, width), zero_if_tiny(upper, width)]);
    }

    let width = pretty_width((max - min) / suggested_bins as f64);
    let lower = (min / width).floor() * width;
    let upper = (max / width).ceil() * width;
    let intervals = ((upper - lower) / width).round().max(1.0) as usize;

    Ok((0..=intervals)
        .map(|i| zero_if_tiny(lower + i as f64 * width, width))
        .collect())
}

/// Rounds a raw cell width to the nearest R-style 1/2/5/10 * 10^k step.
fn pretty_width(cell_width: f64) -> f64 {
    // These cutoffs empirically match R's default pretty() behavior.
    const ONE_CUTOFF: f64 = 1.4;
    const TWO_CUTOFF: f64 = 2.8;
    const FIVE_CUTOFF: f64 = 7.0;

    let base = 10.0_f64.powf(cell_width.log10().floor());
    let unit = cell_width / base;

    let pretty_unit = if unit <= ONE_CUTOFF {
        1.0
    } else if unit <= TWO_CUTOFF {
        2.0
    } else if unit <= FIVE_CUTOFF {
        5.0
    } else {
        10.0
    };

    pretty_unit * base
}

/// Removes tiny floating-point artifacts around zero that can appear after
/// computing pretty break endpoints.
fn zero_if_tiny(value: f64, width: f64) -> f64 {
    if value.abs() < 1e-14 * width.abs().max(1.0) {
        0.0
    } else {
        value
    }
}

/// Counts finite observations into the supplied histogram breaks, including
/// values exactly on the final break in the last bin.
fn histogram_counts(data: &[f64], breaks: &[f64]) -> Vec<f64> {
    let mut counts = vec![0.0; breaks.len().saturating_sub(1)];
    if counts.is_empty() {
        return counts;
    }

    let Some(&lower) = breaks.first() else {
        return counts;
    };
    let Some(&upper) = breaks.last() else {
        return counts;
    };

    for &value in data {
        if !value.is_finite() || value < lower || value > upper {
            continue;
        }

        let upper_break = breaks.partition_point(|breakpoint| *breakpoint < value);
        let bin = upper_break.saturating_sub(1).min(counts.len() - 1);
        counts[bin] += 1.0;
    }

    counts
}
