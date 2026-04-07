use anyhow::Context;
use kuva::{
    prelude::{Layout, Plot},
    render_to_svg,
};
use std::path::Path;

pub mod clustermap;
pub mod heuristics;
pub mod sankey;
pub mod coverage;

pub use heuristics::*;

pub fn render_plots(
    plots: Vec<(String, (Vec<Plot>, Layout))>,
    outpath: impl AsRef<Path>,
) -> anyhow::Result<()> {
    for plot in plots {
        let (filename, (plots, layout)) = plot;
        let filepath = outpath.as_ref().join(filename);
        let svg = render_to_svg(plots, layout);
        std::fs::write(&filepath, svg)
            .with_context(|| format!("Failed to write output file \'{}\'", &filepath.display()))?;
    }

    Ok(())
}
