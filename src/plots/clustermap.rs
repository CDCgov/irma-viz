use crate::{
    config::Config,
    data::SquareMatrix,
    plots::{render_multiplot, render_plot},
};
use anyhow::Result;
use kuva::prelude::*;
use std::sync::Arc;

/// TODO: Docs
pub fn kuva_clustermap(data: SquareMatrix) -> Vec<Plot> {
    let colormap = ColorMap::Custom(Arc::new(|t: f64| {
        let t = t.clamp(0.0, 1.0);

        // light grey: #7b7b7b
        let r1 = 0x7b as f64;
        let g1 = 0x7b as f64;
        let b1 = 0x7b as f64;

        // bright red: #ff0000
        let r0 = 0xff as f64;
        let g0 = 0x00 as f64;
        let b0 = 0x00 as f64;

        let r = r0 + t * (r1 - r0);
        let g = g0 + t * (g1 - g0);
        let b = b0 + t * (b1 - b0);

        format!("rgb({},{},{})", r as u8, g as u8, b as u8)
    }));

    vec![
        Clustermap::new()
            .with_data(data.matrix)
            .with_color_map(colormap)
            .with_row_labels(data.labels.clone())
            .with_col_labels(data.labels)
            .into(),
    ]
}

pub fn plot_clustermap(data: SquareMatrix, cfg: &Config, target: &str) -> Result<()> {
    let plot = kuva_clustermap(data);
    let layout = Layout::auto_from_plots(&plot)
        .with_title(format!("Variant site clusters, {target}-EXPENRD.sqm"));

    let filename = format!("{target}-EXPENRD.svg");
    render_plot((filename.as_str(), (plot, layout)), cfg.output.path.clone())
}

pub fn plot_heat_phylo(data: SquareMatrix, cfg: &Config, target: &str) -> Result<()> {
    let tree_height = cfg.constants.tree_height;
    let line_placement = 1.0 - tree_height * 0.93;

    let (dendrogram, leaf_order) = kuva_dendro(&data);
    let dendro_layout = Layout::auto_from_plots(&dendrogram)
        .with_title("Variant site clusters")
        .with_reference_line(ReferenceLine::vertical(line_placement));

    let (heatmap, layout_cats) = kuva_heatmap(&data, leaf_order);
    let heat_layout = Layout::auto_from_plots(&heatmap)
        .with_title(format!("{target}-EXPENRD.sqm"))
        .with_x_categories(layout_cats.clone().into_iter().rev().collect::<Vec<_>>())
        .with_y_categories(layout_cats);

    let filename = format!("{target}-EXPENRD.svg");
    let scene = Figure::new(1, 2)
        .with_plots(vec![dendrogram, heatmap])
        .with_layouts(vec![dendro_layout, heat_layout])
        .render();

    render_multiplot(&scene, cfg.output.path.clone(), &filename)
}

fn kuva_dendro(data: &SquareMatrix) -> (Vec<Plot>, Vec<String>) {
    let labels = data.labels.iter().map(|l| l.as_str()).collect::<Vec<_>>();
    let dist = &data.matrix;

    let tree = PhyloTree::from_distance_matrix(&labels, dist).with_phylogram();
    let leaf_order = tree.leaf_labels_top_to_bottom();

    (vec![tree.into()], leaf_order)
}

fn kuva_heatmap(data: &SquareMatrix, leaf_order: Vec<String>) -> (Vec<Plot>, Vec<String>) {
    let dist = &data.matrix;

    let colormap = ColorMap::Custom(Arc::new(|t: f64| {
        let t = t.clamp(0.0, 1.0);

        // light grey: #7b7b7b
        let r1 = 0x7b as f64;
        let g1 = 0x7b as f64;
        let b1 = 0x7b as f64;

        // bright red: #ff0000
        let r0 = 0xff as f64;
        let g0 = 0x00 as f64;
        let b0 = 0x00 as f64;

        let r = r0 + t * (r1 - r0);
        let g = g0 + t * (g1 - g0);
        let b = b0 + t * (b1 - b0);

        format!("rgb({},{},{})", r as u8, g as u8, b as u8)
    }));

    let heatmap = Heatmap::new()
        .with_data(dist.clone())
        .with_color_map(colormap)
        .with_labels(data.labels.clone(), data.labels.clone())
        .with_x_categories(leaf_order.clone())
        .with_y_categories(leaf_order);

    let layout_cats = heatmap
        .row_labels
        .clone()
        .expect("Function is not called if Square Matrix is empty.");

    (vec![heatmap.into()], layout_cats)
}
