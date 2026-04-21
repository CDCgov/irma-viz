use anyhow::{Context, Result};
use kuva::{
    prelude::{Layout, Plot, SvgBackend},
    render::render::Scene,
    render_to_svg,
};
use std::path::Path;

pub mod clustermap;
pub mod coverage;
pub mod heuristics;
pub mod read_percentages;

pub fn render_plot(plot: (&str, (Vec<Plot>, Layout)), outpath: impl AsRef<Path>) -> Result<()> {
    let (filename, (plots, layout)) = plot;
    let filepath = outpath.as_ref().join(filename);
    let svg = render_to_svg(plots, layout);
    std::fs::write(&filepath, svg)
        .with_context(|| format!("Failed to write output file \'{}\'", &filepath.display()))?;

    Ok(())
}

pub fn render_multiplot(scene: &Scene, outpath: impl AsRef<Path>, filename: &str) -> Result<()> {
    let filepath = outpath.as_ref().join(filename);
    let svg = SvgBackend.render_scene(scene);

    std::fs::write(&filepath, svg)
        .with_context(|| format!("Failed to write output file \'{}\'", &filepath.display()))?;

    Ok(())
}
