use kuva::{
    plot::PhyloTree,
    prelude::{Layout, Plot},
};

use crate::data::SquareMatrix;

pub fn kuva_tree(data: SquareMatrix) -> (Vec<Plot>, Layout) {
    let tree = PhyloTree::from_newick(&data.newick());

    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots).with_title("Variant Phase Clustering");

    (plots, layout)
}
