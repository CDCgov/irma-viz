use crate::data::ReadCountsData;
use kuva::{
    plot::SankeyPlot,
    prelude::{Layout, Plot},
};

/*pub fn kuva_sankey(data: ReadCountsData) { //(Vec<Plot>, Layout) {
    let sankey_edges: Vec<(&str, &str, f64)> = Vec::new();
    for (key, datum) in data.record_data_map {
        let value = datum._read.unwrap();
        if key.eq("1-initial")
    }
}
*/

pub fn to_sankey_vec(data: &ReadCountsData) -> Vec<(&str, &str, f64)> {
    let mut out = Vec::new();
    let lines = &data.record_data_map;

    let get_read = |key: &str| -> Option<f64> { lines.get(key)?._read.map(|v| v as f64) };

    if let Some(v) = get_read("2-passQC") {
        out.push(("Initial Reads", "Pass QC", v));
    }

    if let Some(v) = get_read("2-failQC") {
        out.push(("Initial Reads", "Fail QC", v));
    }

    if let Some(v) = get_read("3-match") {
        out.push(("Pass QC", "Primary Match", v));
    }

    if let Some(v) = get_read("3-nomatch") {
        out.push(("Pass QC", "No Match", v));
    }

    if let Some(v) = get_read("3-altmatch") {
        out.push(("Pass QC", "Alt Match", v))
    }

    if let Some(v) = get_read("3-chimeric") {
        out.push(("Pass QC", "Chimeric", v));
    }

    for (key, line) in lines.iter() {
        if let Some(target) = key.as_str().strip_prefix("4-")
            && let Some(v) = line._read
        {
            out.push(("Primary Match", target, v as f64));
        } else if let Some(target) = key.as_str().strip_prefix("5-")
            && let Some(v) = line._read
        {
            out.push(("Alt Match", target, v as f64));
        }
    }

    out
}

pub fn kuva_sankey(edges: Vec<(&str, &str, f64)>) -> (Vec<Plot>, Layout) {
    let sankey = SankeyPlot::new().with_links(edges);
    let plots = vec![sankey.into()];
    let layout = Layout::auto_from_plots(&plots);
    (plots, layout)
}
