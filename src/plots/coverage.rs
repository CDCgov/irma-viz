use crate::data::{Coverage, Variants};
use kuva::prelude::*;

pub fn kuva_coverage(coverage: Coverage, variants: Variants) -> (Vec<Plot>, Layout) {
    if variants.data.is_empty() {
        let min_y = coverage
            .coverage_by_position
            .iter()
            .map(|&(_, pos)| pos)
            .min_by(|a, b| a.total_cmp(b))
            .expect("Data is checked for empty and Nones at import.");

        let line = LinePlot::new()
            .with_data(coverage.coverage_by_position)
            .with_fill()
            .with_stroke_width(0.3);

        let plots = vec![line.into()];
        let layout = Layout::auto_from_plots(&plots)
            .with_y_axis_min(min_y - min_y * 0.05)
            .with_y_label("Coverage depth")
            .with_x_label("<gene> position");

        (plots, layout)
    } else {
        todo!();
    }
}
