use crate::data::SquareMatrix;
use kuva::prelude::*;

/// TODO: Docs
pub fn kuva_clustermap(data: SquareMatrix) -> (Vec<Plot>, Layout) {
    let clustermap = Clustermap::new()
        .with_data(data.matrix)
        .with_row_labels(data.labels.clone())
        .with_col_labels(data.labels);

    let plots = vec![Plot::Clustermap(clustermap)];
    let layout = Layout::auto_from_plots(&plots).with_title("Clustermap Title");
    (plots, layout)
}
