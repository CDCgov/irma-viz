use anyhow::{Context, Result};
use kuva::{
    PdfBackend,
    prelude::{Layout, Plot, SvgBackend},
    render::render::Scene,
    render_to_pdf, render_to_svg,
};
use std::path::Path;

use crate::config::OutputFormat;

pub mod clustermap;
pub mod coverage;
pub mod heuristics;
pub mod read_percentages;

pub fn render_plot(
    plot: (&str, (Vec<Plot>, Layout)),
    outpath: impl AsRef<Path>,
    output_format: OutputFormat,
) -> Result<()> {
    let (filename, (plots, layout)) = plot;
    let filepath = outpath
        .as_ref()
        .join(format!("{filename}{}", output_format));

    match output_format {
        OutputFormat::Pdf => {
            let pdf = render_to_pdf(plots, layout)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("Failed to render PDF '{}'", filepath.display()))?;
            std::fs::write(&filepath, pdf).with_context(|| {
                format!("Failed to write output file \'{}\'", filepath.display())
            })?;
        }
        OutputFormat::Svg => {
            let svg = render_to_svg(plots, layout);
            std::fs::write(&filepath, svg).with_context(|| {
                format!("Failed to write output file \'{}\'", filepath.display())
            })?;
        }
    }

    Ok(())
}

pub fn render_multiplot(
    scene: &Scene,
    outpath: impl AsRef<Path>,
    filename: &str,
    output_format: OutputFormat,
) -> Result<()> {
    let filepath = outpath.as_ref().join(format!("{filename}{output_format}"));

    match output_format {
        OutputFormat::Pdf => {
            let pdf = PdfBackend::new()
                .render_scene(scene)
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("Failed to render PDF '{}'", filepath.display()))?;
            std::fs::write(&filepath, pdf).with_context(|| {
                format!("Failed to write output file \'{}\'", filepath.display())
            })?;
        }
        OutputFormat::Svg => {
            let svg = SvgBackend.render_scene(scene);
            std::fs::write(&filepath, svg).with_context(|| {
                format!("Failed to write output file \'{}\'", filepath.display())
            })?;
        }
    }

    Ok(())
}
