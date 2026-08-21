//! Demo figure generation, available only with the `demo` feature.

use crate::{
    config::{
        CoverageColorOption, MatrixType, OutputFormat, ParsedConfig, PercentVizOption,
        discover_clustermap_targets, is_valid_target_name,
    },
    diagnostics::PlotError,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

// list of matrix types to check for creating the demo
const MATRIX_TYPES: [MatrixType; 4] = [
    MatrixType::Expenrd,
    MatrixType::Jaccard,
    MatrixType::Mutuald,
    MatrixType::Njointp,
];

// constants for creating the combined svg
const COMBINED_COLS: usize = 3;
const COMBINED_GAP: f64 = 24.0;
const COMBINED_PADDING: f64 = 24.0;
const COMBINED_CELL_WIDTH: f64 = 480.0;
const COMBINED_ROW_HEIGHTS: [f64; 2] = [420.0, 420.0];
const SVG_XMLNS: &str = "http://www.w3.org/2000/svg";

/// Generates the SVG demo set and `combined.svg` for one target.
///
/// Creates read-percentages (pie and sankey) coverage (nucleotide and
/// frequency), clustermap (tree and heatmap) variants.
pub fn run_demo(cfg: &mut ParsedConfig, target: &str) -> Result<(), PlotError> {
    if !is_valid_target_name(target) {
        return Err(PlotError::ConfigError(format!(
            "invalid target name for demo_target: '{target}'"
        )));
    }

    cfg.io_args.output_format = OutputFormat::Svg;

    render_demo_read_percentages(cfg)?;
    render_demo_heuristics(cfg, target)?;
    render_demo_coverage(cfg, target)?;
    let matrix_name = render_demo_clustermap(cfg, target)?;

    render_combined_demo_svg(&cfg.io_args.output_path, target, matrix_name)?;

    Ok(())
}

fn render_demo_read_percentages(cfg: &mut ParsedConfig) -> Result<(), PlotError> {
    cfg.plot_specific.read_percent.viz_option = PercentVizOption::Sankey;
    crate::run_read_percentages(cfg)?;
    rename_demo_output(
        cfg.io_args.output_path.as_path(),
        "READ_PERCENTAGES.svg",
        "READ_PERCENTAGES_sankey.svg",
    )?;

    // Render both demo variants while keeping the standard pie-chart filename.
    cfg.plot_specific.read_percent.viz_option = PercentVizOption::Pie(true);
    crate::run_read_percentages(cfg)?;

    Ok(())
}

fn render_demo_heuristics(cfg: &ParsedConfig, target: &str) -> Result<(), PlotError> {
    crate::run_heuristics_for_target(cfg, target)
}

fn render_demo_coverage(cfg: &mut ParsedConfig, target: &str) -> Result<(), PlotError> {
    let original_color = cfg.plot_specific.coverage.color_option;
    cfg.plot_specific.coverage.color_option = CoverageColorOption::Frequency;
    crate::run_coverage_for_target(cfg, target)?;
    rename_demo_output(
        cfg.io_args.output_path.as_path(),
        &format!("{target}-coverageDiagram.svg"),
        &format!("{target}-coverageDiagram_frequency.svg"),
    )?;

    cfg.plot_specific.coverage.color_option = CoverageColorOption::Nucleotide;
    let render_result = crate::run_coverage_for_target(cfg, target);
    cfg.plot_specific.coverage.color_option = original_color;
    render_result
}

fn render_demo_clustermap(cfg: &mut ParsedConfig, target: &str) -> Result<&'static str, PlotError> {
    let cluster_targets = discover_clustermap_targets(cfg)?;
    let matrix_type = MATRIX_TYPES
        .into_iter()
        .find(|matrix_type| cluster_targets.targets_for(*matrix_type).contains(target))
        .ok_or_else(|| {
            PlotError::MissingData(format!(
                "no clustermap matrix file was found for target {target}"
            ))
        })?;

    let matrix_name = matrix_type.display_name();
    let original_option = cfg.plot_specific.cluster_config.cluster_option;

    cfg.plot_specific.cluster_config.cluster_option = crate::config::ClusterOption::Tree;
    crate::run_clustermap_for_matrix_type(cfg, target, matrix_type)?;
    rename_demo_output(
        cfg.io_args.output_path.as_path(),
        &format!("{target}-{matrix_name}.svg"),
        &format!("{target}-{matrix_name}_tree.svg"),
    )?;

    cfg.plot_specific.cluster_config.cluster_option = crate::config::ClusterOption::Clustermap;
    let render_result = crate::run_clustermap_for_matrix_type(cfg, target, matrix_type);
    cfg.plot_specific.cluster_config.cluster_option = original_option;
    render_result?;

    Ok(matrix_name)
}

fn rename_demo_output(output_path: &Path, from: &str, to: &str) -> Result<(), PlotError> {
    fs::rename(output_path.join(from), output_path.join(to))
        .map_err(|err| PlotError::IOError(format!("renaming '{from}' to '{to}'"), err))
}

/// SVG body and dimensions for one panel in the combined demo figure.
#[derive(Debug)]
struct SvgPanel {
    pub width: f64,
    pub height: f64,
    // holds attributes from the original svgs from kuva, namely fonts
    pub root_attrs: String,
    pub body: String,
}

/// Loads rendered SVG panels and stitches them into `combined.svg`.
fn render_combined_demo_svg(
    output_path: &Path,
    target: &str,
    matrix_name: &str,
) -> Result<(), PlotError> {
    // read all svgs
    let panels = [
        load_svg_panel(output_path.join("READ_PERCENTAGES.svg"), "combined-pies")?,
        load_svg_panel(
            output_path.join(format!("{target}-heuristics.svg")),
            "combined-heuristics",
        )?,
        load_svg_panel(
            output_path.join("READ_PERCENTAGES_sankey.svg"),
            "combined-sankey",
        )?,
        load_svg_panel(
            output_path.join(format!("{target}-{matrix_name}.svg")),
            "combined-clustermap",
        )?,
        load_svg_panel(
            output_path.join(format!("{target}-coverageDiagram.svg")),
            "combined-coverage",
        )?,
        load_svg_panel(
            output_path.join(format!("{target}-{matrix_name}_tree.svg")),
            "combined-tree",
        )?,
    ];

    // layout of the plots
    let positions = [(0usize, 0usize), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)];

    // total dimensions of the finished svgs
    let total_width = COMBINED_PADDING * 2.0
        + COMBINED_CELL_WIDTH * COMBINED_COLS as f64
        + COMBINED_GAP * (COMBINED_COLS.saturating_sub(1)) as f64;
    let total_height = COMBINED_PADDING * 2.0
        + COMBINED_ROW_HEIGHTS.iter().sum::<f64>()
        + COMBINED_GAP * (COMBINED_ROW_HEIGHTS.len().saturating_sub(1)) as f64;

    let mut svg = String::new();
    // xmlns is a namespace so browsers parse the combined file as SVG instead
    // of generic XML.
    svg.push_str(&format!(
        "<svg xmlns=\"{SVG_XMLNS}\" width=\"{total_width}\" height=\"{total_height}\" viewBox=\"0 0 {total_width} {total_height}\">"
    ));
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"white\" />");

    // add the panels into their positions on the final combined plot
    for (panel, (row, col)) in panels.into_iter().zip(positions) {
        let x = COMBINED_PADDING + col as f64 * (COMBINED_CELL_WIDTH + COMBINED_GAP);
        let y = COMBINED_PADDING
            + COMBINED_ROW_HEIGHTS.iter().take(row).sum::<f64>()
            + row as f64 * COMBINED_GAP;
        let cell_height = COMBINED_ROW_HEIGHTS[row];

        svg.push_str(&format!(
            "<svg x=\"{x}\" y=\"{y}\" width=\"{COMBINED_CELL_WIDTH}\" height=\"{cell_height}\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\"{}>{}</svg>",
            panel.width, panel.height, panel.root_attrs, panel.body
        ));
    }

    svg.push_str("</svg>");

    fs::write(output_path.join("combined.svg"), svg)
        .map_err(|err| PlotError::IOError("writing 'combined.svg'".to_string(), err))?;

    Ok(())
}

/// Reads in an svg's data and metadata
fn load_svg_panel(path: PathBuf, id_prefix: &str) -> Result<SvgPanel, PlotError> {
    let svg = fs::read_to_string(&path)
        .map_err(|err| PlotError::IOError(format!("reading '{}'", path.display()), err))?;
    let open_start = svg.find("<svg").ok_or_else(|| {
        PlotError::InvalidData(format!("no <svg> tag found in '{}'", path.display()))
    })?;
    let open_end = svg[open_start..]
        .find('>')
        .map(|idx| open_start + idx)
        .ok_or_else(|| {
            PlotError::InvalidData(format!("malformed <svg> tag in '{}'", path.display()))
        })?;
    let root_tag = &svg[open_start..=open_end];
    let close_start = svg.rfind("</svg>").ok_or_else(|| {
        PlotError::InvalidData(format!(
            "no closing </svg> tag found in '{}'",
            path.display()
        ))
    })?;

    let width = parse_svg_dimension(root_tag, "width")
        .map_err(|err| PlotError::InvalidData(format!("{err} in '{}'", path.display())))?;
    let height = parse_svg_dimension(root_tag, "height")
        .map_err(|err| PlotError::InvalidData(format!("{err} in '{}'", path.display())))?;
    let root_attrs = extract_root_attrs(root_tag);

    let body =
        svg[open_end + 1..close_start].replace("kuva-clip-", &format!("{id_prefix}-kuva-clip-"));

    Ok(SvgPanel {
        width,
        height,
        root_attrs,
        body,
    })
}

/// Parses a numeric SVG dimension with an optional unit suffix.
fn parse_svg_dimension(svg_tag: &str, attribute: &str) -> Result<f64, PlotError> {
    let marker = format!("{attribute}=\"");
    let start = svg_tag
        .find(&marker)
        .map(|idx| idx + marker.len())
        .ok_or_else(|| PlotError::InvalidData(format!("attribute {attribute:?} not found")))?;
    let end = svg_tag[start..]
        .find('"')
        .map(|idx| start + idx)
        .ok_or_else(|| PlotError::InvalidData(format!("attribute {attribute:?} is not closed")))?;
    let raw = &svg_tag[start..end];
    let numeric = raw.trim_end_matches(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-')));

    numeric
        .parse::<f64>()
        .map_err(|_| PlotError::InvalidData(format!("invalid {attribute:?} value {raw:?}")))
}

/// Extracts root attributes (specifically font) inherited by an embedded SVG
/// panel.
fn extract_root_attrs(root_tag: &str) -> String {
    let mut attrs = String::new();

    for key in ["font-family", "fill"] {
        if let Some(value) = extract_svg_attr(root_tag, key) {
            attrs.push(' ');
            attrs.push_str(key);
            attrs.push_str("=\"");
            attrs.push_str(&value);
            attrs.push('"');
        }
    }

    attrs
}

/// Extracts a quoted attribute value from an SVG tag.
fn extract_svg_attr(svg_tag: &str, attribute: &str) -> Option<String> {
    let marker = format!("{attribute}=\"");
    let start = svg_tag.find(&marker)? + marker.len();
    let end = svg_tag[start..].find('"')? + start;
    Some(svg_tag[start..end].to_owned())
}
