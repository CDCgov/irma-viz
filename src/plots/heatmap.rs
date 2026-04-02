use crate::data::SquareMatrix;
use kuva::{plot::Heatmap, prelude::*};
use std::collections::HashMap;

/// TODO: Docs
pub fn kuva_heatmap(mut data: SquareMatrix) -> (Vec<Plot>, Layout) {
    let tree = PhyloTree::from_distance_matrix(
        &data.labels.iter().map(String::as_str).collect::<Vec<_>>(),
        &data.matrix,
    );
    let leaf_order = tree.leaf_labels_top_to_bottom();

    let index_by_label = data
        .labels
        .iter()
        .enumerate()
        .map(|(idx, label)| (label.as_str(), idx))
        .collect::<HashMap<_, _>>();
    let order = leaf_order
        .iter()
        .map(|label| {
            *index_by_label
                .get(label.as_str())
                .expect("Tree leaf label should exist in matrix labels")
        })
        .collect::<Vec<_>>();

    data.labels = order.iter().map(|&idx| data.labels[idx].clone()).collect();
    data.matrix = order
        .iter()
        .map(|&row_idx| {
            order
                .iter()
                .map(|&col_idx| data.matrix[row_idx][col_idx])
                .collect()
        })
        .collect();
    let mut y_labels = data.labels.clone();
    y_labels.reverse();
    data.matrix.reverse();

    let heatmap = Heatmap::new().with_data(data.matrix);
    let plots: Vec<Plot> = vec![heatmap.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Site Clusters")
        .with_x_categories(data.labels)
        .with_y_categories(y_labels);

    (plots, layout)
}
