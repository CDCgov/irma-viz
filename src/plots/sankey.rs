use kuva::{
    plot::SankeyPlot,
    prelude::{Layout, Plot},
};

pub fn kuva_sankey(edges: Vec<(String, String, f64)>) -> (Vec<Plot>, Layout) {
    let sankey = SankeyPlot::new().with_links(edges);
    let plots = vec![sankey.into()];
    let layout = Layout::auto_from_plots(&plots);
    (plots, layout)
}
