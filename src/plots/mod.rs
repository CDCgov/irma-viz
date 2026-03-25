pub mod heuristics;
pub mod sankey;

use std::path::{Path};

use kuva::{prelude::{Layout, Plot}, render_to_svg};


pub fn render_plots(plots: Vec<(String, (Vec<Plot>, Layout))>, outpath: impl AsRef<Path>) {
    for plot in plots {
        let (filename, (plots, layout)) = plot;
        let filepath = outpath.as_ref().join(filename);
        let svg = render_to_svg(plots, layout);
        std::fs::write(filepath, svg).unwrap();
    }
}


pub use heuristics::*;
