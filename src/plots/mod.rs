//! Figure construction and rendering for IRMA output data.

use kuva::{
    prelude::{Layout, Plot, SvgBackend},
    render::render::Scene,
    render_to_svg,
};
use std::path::Path;

#[cfg(feature = "pdf")]
use kuva::{PdfBackend, render_to_pdf};

use crate::{config::OutputFormat, diagnostics::PlotError};

pub mod clustermap;
pub mod coverage;
pub mod heuristics;
pub mod read_percentages;

/// Renders a single kuva plot to `{outpath}/{filename}{output_format}`
///
/// ## Errors
///
/// Can return either a [`PlotError::RenderError`] if there is an issue with
/// kuva converting from svg to pdf, or a [`PlotError::IOError`] if there is an
/// issue writing to file.
pub fn render_plot(
    plot: (&str, (Vec<Plot>, Layout)),
    outpath: impl AsRef<Path>,
    output_format: OutputFormat,
) -> Result<(), PlotError> {
    let (filename, (plots, layout)) = plot;
    let filepath = outpath
        .as_ref()
        .join(format!("{filename}{}", output_format));

    cfg_select! {
        feature = "pdf" => match output_format {
            OutputFormat::Pdf => {
                let pdf = render_to_pdf(plots, layout).map_err(|err| {
                    PlotError::RenderError(format!("render PDF '{}': {err}", filepath.display()))
                })?;
                std::fs::write(&filepath, pdf).map_err(|err| {
                    PlotError::IOError(format!("writing '{}'", filepath.display()), err)
                })?;
            }
            OutputFormat::Svg => {
                let svg = render_to_svg(plots, layout);
                std::fs::write(&filepath, svg).map_err(|err| {
                    PlotError::IOError(format!("writing '{}'", filepath.display()), err)
                })?;
            }
        },
        _ => {
            let svg = render_to_svg(plots, layout);
            std::fs::write(&filepath, svg).map_err(|err| {
                PlotError::IOError(format!("writing '{}'", filepath.display()), err)
            })?;
        }
    }

    Ok(())
}

/// Renders a figure with multiple subplots from a [`Scene`] to
/// `{outpath}/{filename}{output_format}`.
///
/// ## Errors
///
/// Can return either a [`PlotError::RenderError`] if there is an issue with
/// kuva converting from svg to pdf, or a [`PlotError::IOError`] if there is an
/// issue writing to file.
pub fn render_multiplot(
    scene: &Scene,
    outpath: impl AsRef<Path>,
    filename: &str,
    output_format: OutputFormat,
) -> Result<(), PlotError> {
    let filepath = outpath.as_ref().join(format!("{filename}{output_format}"));

    cfg_select! {
        feature = "pdf" => match output_format {
            OutputFormat::Pdf => {
                let pdf = PdfBackend::new().render_scene(scene).map_err(|err| {
                    PlotError::RenderError(format!("render PDF '{}': {err}", filepath.display()))
                })?;
                std::fs::write(&filepath, pdf).map_err(|err| {
                    PlotError::IOError(format!("writing '{}'", filepath.display()), err)
                })?;
            }
            OutputFormat::Svg => {
                let svg = SvgBackend.render_scene(scene);
                std::fs::write(&filepath, svg).map_err(|err| {
                    PlotError::IOError(format!("writing '{}'", filepath.display()), err)
                })?;
            }
        },
        _ => {
            let svg = SvgBackend.render_scene(scene);
            std::fs::write(&filepath, svg).map_err(|err| {
                PlotError::IOError(format!("writing '{}'", filepath.display()), err)
            })?;
        }
    }

    Ok(())
}
