use crate::{config::Config, data::SankeyVec, plots::render_plot};
use anyhow::Result;
use kuva::{
    plot::SankeyPlot,
    prelude::{Layout, Plot},
};

pub fn kuva_sankey(sankey_vec: SankeyVec) -> (Vec<Plot>, Layout) {
    let sankey = SankeyPlot::new()
        .with_links(sankey_vec.edges)
        .with_flow_labels()
        .with_flow_label_min_height(0.0);
    let plots = vec![sankey.into()];
    let layout = Layout::auto_from_plots(&plots);
    (plots, layout)
}

pub fn plot_sankey(sankey_vec: SankeyVec, cfg: &Config) -> Result<()> {
    let (plot, layout) = kuva_sankey(sankey_vec);

    render_plot(("READ_PERCENTAGES.svg", (plot, layout)), cfg.output.path.clone())
}
