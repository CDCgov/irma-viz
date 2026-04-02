use kuva::{
    plot::SankeyPlot,
    prelude::{Layout, Plot},
};

pub fn kuva_sankey(edges: Vec<(String, String, f64)>) -> (Vec<Plot>, Layout) {
    let sankey = SankeyPlot::new()
        .with_links(edges)
        .with_flow_labels()
        .with_flow_label_min_height(0.0);
    let plots = vec![sankey.into()];
    let layout = Layout::auto_from_plots(&plots);
    (plots, layout)
}
