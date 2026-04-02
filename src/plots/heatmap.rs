use crate::data::SquareMatrix;
use kuva::{plot::Heatmap, prelude::*};

/// TODO: Docs
pub fn kuva_heatmap(mut data: SquareMatrix) -> (Vec<Plot>, Layout) {
    // Replicate original orientation (flip y-axis from kuva default)
    data.matrix.reverse();
    let x_categories = data.labels.clone();
    data.labels.reverse();

    let heatmap = Heatmap::new().with_data(data.matrix);
    let plots: Vec<Plot> = vec![heatmap.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Variant Site Clusters")
        .with_x_categories(x_categories)
        .with_y_categories(data.labels);

    (plots, layout)
}
