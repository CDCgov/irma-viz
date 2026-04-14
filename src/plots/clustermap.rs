use crate::{config::Config, data::SquareMatrix, plots::render_plot};
use anyhow::Result;
use kuva::prelude::*;

/// TODO: Docs
pub fn kuva_clustermap(data: SquareMatrix) -> (Vec<Plot>, Layout) {
    let clustermap = Clustermap::new()
        .with_data(data.matrix)
        .with_row_labels(data.labels.clone())
        .with_col_labels(data.labels);

    let plot = vec![Plot::Clustermap(clustermap)];
    let layout = Layout::auto_from_plots(&plot).with_title("Clustermap Title");
    (plot, layout)
}

pub fn plot_clustermap(data: SquareMatrix, cfg: &Config, target: &String) -> Result<()> {
    let (plot, layout) = kuva_clustermap(data);

    let filename = format!("{target}-EXPENRD.svg");
    render_plot((filename.as_str(), (plot, layout)), cfg.output.path.clone())
}
