use crate::{config::Config, data::AllAlleles, plots::render_multiplot};
use anyhow::{Context, Result};
use kuva::{plot::Histogram, prelude::*};

const NUM_BINS: usize = 50; // from IRMA
const SAMPLES: usize = 1000;

pub fn plot_heuristics(all_alleles: AllAlleles, cfg: &Config, target: &str) -> Result<()> {
    // constants
    let min_aq = cfg.constants.min_aq;
    let min_f = cfg.constants.min_f;
    let min_tcc = cfg.constants.min_tcc;
    let min_conf = cfg.constants.min_conf;

    // Average allele quality density
    let average_qualities = all_alleles.average_qualities;
    if average_qualities.data.is_empty() {
        anyhow::bail!("Density plot has no data");
    }
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

    // Limited x range average allele quality density
    let (limited_aq_density, min_y, max_y) =
        kuva_dens(&average_qualities.data, average_qualities.min, min_aq);
    let lim_aq_dens_layout = Layout::auto_from_plots(&limited_aq_density)
        .with_title(format!("to {min_aq}"))
        .with_x_axis_min(average_qualities.min)
        .with_x_axis_max(min_aq)
        .with_y_axis_min(min_y - min_y * 0.05)
        .with_y_axis_max(max_y + max_y * 0.05)
        .with_show_grid(false);

    // Observed frequency density
    let frequencies = all_alleles.frequencies;
    let (freq_density, min_y, max_y) = kuva_dens(&frequencies, 0.0, 0.1);
    let freq_dens_layout = Layout::auto_from_plots(&freq_density)
        .with_title("Density of observed frequency (to 10%)")
        .with_x_axis_min(0.0)
        .with_x_axis_max(0.1)
        .with_y_axis_min(min_y - min_y * 0.05)
        .with_y_axis_max(max_y + max_y * 0.05)
        .with_reference_line(ReferenceLine::vertical(min_f).with_dasharray("none"))
        .with_show_grid(false);

    // Limited x range observed frequency density
    let (lim_freq_dens, min_y, max_y) = kuva_dens(&frequencies, 0.0, min_f);
    let lim_freq_dens_layout = Layout::auto_from_plots(&lim_freq_dens)
        .with_title(format!("to {min_f}"))
        .with_x_axis_min(0.0)
        .with_x_axis_max(min_f)
        .with_y_axis_min(min_y - min_y * 0.001)
        .with_y_axis_max(max_y + max_y * 0.001)
        .with_show_grid(false);

    // Coverage histogram
    let cov_hist = kuva_histogram(all_alleles.totals.data, NUM_BINS)
        .with_context(|| "coverage histogram subplot")?;
    let cov_hist_layout = Layout::auto_from_plots(&cov_hist)
        .with_x_axis_min(0.0)
        .with_x_axis_max(all_alleles.totals.upper_quantile + 1.0)
        .with_reference_line(ReferenceLine::vertical(min_tcc).with_dasharray("none"))
        .with_show_grid(false)
        .with_title("Histogram of coverage (Depth <= 20% Quantile)");

    // Machine error confidence histogram
    let confidence_values = all_alleles.confidence_not_mac_errs;
    let confidence_histogram = kuva_histogram(confidence_values, NUM_BINS)
        .with_context(|| "confidence histogram subplot")?;
    let confidence_hist_layout = Layout::auto_from_plots(&confidence_histogram)
        .with_reference_line(ReferenceLine::vertical(min_conf).with_dasharray("none"))
        .with_show_grid(false)
        .with_title("Histogram of confidence of not machine error, non-zero");

    // Multi-Plot
    let scene = Figure::new(3, 2)
        .with_plots(vec![
            aq_density,
            limited_aq_density,
            freq_density,
            lim_freq_dens,
            cov_hist,
            confidence_histogram,
        ])
        .with_layouts(vec![
            aq_dens_layout,
            lim_aq_dens_layout,
            freq_dens_layout,
            lim_freq_dens_layout,
            cov_hist_layout,
            confidence_hist_layout,
        ])
        .render();

    let filename = format!("{target}-heuristics.svg");
    render_multiplot(&scene, cfg.output_path()?, filename.as_str())
}

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
fn kuva_histogram(data: Vec<f64>, num_bins: usize) -> Result<Vec<Plot>> {
    if data.is_empty() {
        anyhow::bail!("Histogram plot has no data");
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
fn pretty_breaks(data: &[f64], suggested_bins: usize) -> Result<Vec<f64>> {
    if suggested_bins == 0 {
        anyhow::bail!("Suggested histogram bin count must be greater than zero");
    }

    let (min, max) = data
        .iter()
        .filter(|x| x.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &x| {
            (min.min(x), max.max(x))
        });

    if !min.is_finite() || !max.is_finite() {
        anyhow::bail!("Histogram breaks require at least one finite value");
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
