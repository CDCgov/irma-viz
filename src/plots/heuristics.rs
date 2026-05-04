use crate::{config::Config, data::AllAlleles, plots::render_multiplot};
use anyhow::{Context, Result};
use kuva::{plot::Histogram, prelude::*};

const NUM_BINS: usize = 50; // from IRMA

pub fn kuva_histogram(data: Vec<f64>, num_bins: usize) -> Result<Vec<Plot>> {
    if data.is_empty() {
        anyhow::bail!("Histogram plot has no data");
    }

    let (&first, rest) = data
        .split_first()
        .expect("Already checked if data is empty.");

    let (min, max) = rest.iter().copied().fold((first, first), |(min, max), x| {
        let min = if x.total_cmp(&min).is_lt() { x } else { min };
        let max = if x.total_cmp(&max).is_gt() { x } else { max };
        (min, max)
    });

    Ok(vec![
        Histogram::new()
            .with_data(data)
            .with_bins(num_bins)
            .with_range((min, max))
            .into(),
    ])
}

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
    let aq_density = vec![
        DensityPlot::new()
            .with_data(average_qualities.data.clone())
            .with_x_range(average_qualities.min, average_qualities.max)
            .into(),
    ];
    let aq_dens_layout = Layout::auto_from_plots(&aq_density)
        .with_title("Density of average allele quality")
        .with_x_axis_min(average_qualities.min)
        .with_x_axis_max(average_qualities.max)
        .with_reference_line(ReferenceLine::vertical(min_aq));

    // Limited x range average allele quality density
    let limited_aq_density = vec![
        DensityPlot::new()
            .with_data(average_qualities.data)
            .with_x_range(average_qualities.min, min_aq)
            .into(),
    ];
    let lim_aq_dens_layout = Layout::auto_from_plots(&limited_aq_density)
        .with_title(format!("to {min_aq}"))
        .with_x_axis_min(average_qualities.min)
        .with_x_axis_max(min_aq);

    // Observed frequency density
    let frequencies = all_alleles.frequencies;
    let freq_density = vec![
        DensityPlot::new()
            .with_data(frequencies.clone())
            .with_x_range(0.0, 0.1)
            .into(),
    ];
    let freq_dens_layout = Layout::auto_from_plots(&freq_density)
        .with_title("Density of observed frequency (to 10%)")
        .with_x_axis_min(0.0)
        .with_x_axis_max(0.1)
        .with_reference_line(ReferenceLine::vertical(min_f));

    // Limited x range observed frequency density
    let lim_freq_dens = vec![
        DensityPlot::new()
            .with_data(frequencies)
            .with_x_range(0.0, min_f)
            .into(),
    ];
    let lim_freq_dens_layout = Layout::auto_from_plots(&lim_freq_dens)
        .with_title(format!("to {min_f}"))
        .with_x_axis_min(0.0)
        .with_x_axis_max(min_f);

    // Coverage histogram
    let coverage_histogram = kuva_histogram(all_alleles.totals, NUM_BINS)
        .with_context(|| "coverage histogram subplot")?;
    let cov_hist_layout = Layout::auto_from_plots(&coverage_histogram)
        .with_reference_line(ReferenceLine::vertical(min_tcc))
        .with_title("Histogram of coverage");

    // Machine error confidence histogram
    let confidence_values = all_alleles
        .confidence_not_mac_errs
        .into_iter()
        .flatten()
        // TODO: Check data filtering in IRMA scripts
        .filter(|x| *x != 0.0)
        .collect::<Vec<_>>();
    let confidence_histogram = kuva_histogram(confidence_values, NUM_BINS)
        .with_context(|| "confidence histogram subplot")?;
    let confidence_hist_layout = Layout::auto_from_plots(&confidence_histogram)
        .with_reference_line(ReferenceLine::vertical(min_conf))
        .with_title("Histogram of confidence of not machine error, non-zero");

    // Multi-Plot
    let scene = Figure::new(3, 2)
        .with_plots(vec![
            aq_density,
            limited_aq_density,
            freq_density,
            lim_freq_dens,
            coverage_histogram,
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
    render_multiplot(&scene, cfg.output.path.clone(), filename.as_str())
}
