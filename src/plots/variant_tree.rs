use kuva::{
    plot::PhyloTree,
    prelude::{Layout, Plot},
};

use crate::data::SquareMatrix;

pub fn kuva_tree(data: SquareMatrix) -> (Vec<Plot>, Layout) {
    let tree = PhyloTree::from_distance_matrix(
        &data.labels.iter().map(String::as_str).collect::<Vec<_>>(),
        &data.matrix,
    )
    .with_phylogram();

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots).with_title("Variant Phase Clustering");

    (plots, layout)
}
