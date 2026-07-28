use crate::{
    config::{MatrixType, ParsedConfig, get_directory_paths, is_valid_target_name},
    data::{AllAlleles, AllVariants, Coverage, PairingStats, ReadCounts, SankeyVec, SquareMatrix},
    plots::{
        clustermap::{plot_clustermap, plot_heat_phylo},
        coverage::plot_coverage,
        heuristics::plot_heuristics,
        read_percentages::{plot_perc_pies, plot_perc_sankey},
    },
};
use anyhow::{Context, Result, bail};
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

pub fn run_demo(cfg: &mut ParsedConfig, target: &str) -> Result<()> {
    if !is_valid_target_name(target) {
        bail!("Error: Invalid target name for demo_target");
    }
    let (table_path, matrix_path) = get_directory_paths(&cfg.io_args.input_root);

    // get data for read counts
    let read_counts_path = table_path.join("READ_COUNTS.txt");
    let sankey_vec = SankeyVec::import_from_file(&read_counts_path).with_context(|| {
        format!(
            "Failed to import Read Counts data from '{}'",
            read_counts_path.display()
        )
    })?;
    plot_perc_sankey(sankey_vec, cfg)
        .with_context(|| "Error plotting READ_PERCENTAGES_sankey.svg")?;
    // rename first plot for the purpose of the demo, since we are creating two
    // READ_PERCENTAGES plots and don't want the pie charts to get overridden
    fs::rename(
        cfg.io_args.output_path.join("READ_PERCENTAGES.svg"),
        cfg.io_args.output_path.join("READ_PERCENTAGES_sankey.svg"),
    )
    .with_context(|| "Error renaming READ_PERCENTAGES.svg to READ_PERCENTAGES_sankey.svg")?;

    // create pie charts
    let read_counts = ReadCounts::import_from_file(&read_counts_path).with_context(|| {
        format!(
            "Failed to import Read Counts data from '{}'",
            read_counts_path.display()
        )
    })?;
    plot_perc_pies(read_counts, cfg).with_context(|| "Error plotting READ_PERCENTAGES.svg")?;

    // heuristics
    let all_alleles_path = table_path.join(format!("{target}-allAlleles.txt"));
    let allele_data = AllAlleles::import_from_file(&all_alleles_path).with_context(|| {
        format!(
            "Failed to import All Alleles data from '{}'",
            all_alleles_path.display()
        )
    })?;
    plot_heuristics(allele_data, cfg, target)
        .with_context(|| format!("Error plotting {target}-heuristics.svg"))?;

    // variants and coverage
    let variants_path = table_path.join(format!("{target}-variants.txt"));
    let variants = AllVariants::import_from_file(&variants_path).with_context(|| {
        format!(
            "Failed to import Variants data from '{}'",
            variants_path.display()
        )
    })?;

    let coverage_path = table_path.join(format!("{target}-coverage.txt"));
    let coverage = Coverage::import_from_file(&coverage_path).with_context(|| {
        format!(
            "Failed to import Coverage data from '{}'",
            coverage_path.display()
        )
    })?;

    let pairing_stats_path = table_path.join(format!("{target}-pairingStats.txt"));
    let pairing_stats = PairingStats::import_from_file(&pairing_stats_path).with_context(|| {
        format!(
            "Failed to import Pairing Stats data from '{}'",
            pairing_stats_path.display()
        )
    })?;

    // create frequency coverage plot
    cfg.plot_specific.coverage.color_option = crate::config::CoverageColorOption::Frequency;

    let nucleotide_plot_path = format!("{target}-coverageDiagram.svg");
    let frequency_plot_path = format!("{target}-coverageDiagram_frequency.svg");

    plot_coverage(coverage, variants.clone(), pairing_stats, cfg, target)
        .with_context(|| format!("Error plotting demo frequency plot to {nucleotide_plot_path}"))?;
    fs::rename(
        cfg.io_args.output_path.join(&nucleotide_plot_path),
        cfg.io_args.output_path.join(&frequency_plot_path),
    )
    .with_context(|| format!("Error renaming {nucleotide_plot_path} to {frequency_plot_path}"))?;

    // create nucleotide coverage plot
    cfg.plot_specific.coverage.color_option = crate::config::CoverageColorOption::Nucleotide;
    let coverage = Coverage::import_from_file(&coverage_path).with_context(|| {
        format!(
            "Failed to re-import Coverage data from '{}'",
            coverage_path.display()
        )
    })?;
    let pairing_stats = PairingStats::import_from_file(&pairing_stats_path).with_context(|| {
        format!(
            "Failed to re-import Pairing Stats data from '{}'",
            pairing_stats_path.display()
        )
    })?;
    plot_coverage(coverage, variants.clone(), pairing_stats, cfg, target).with_context(|| {
        format!("Error plotting demo nucleotide plot to {nucleotide_plot_path}")
    })?;

    if variants.positions.len() <= 1 {
        bail!("Demo clustermap output requires more than one variant for target {target}");
    }

    //  clustermaps
    let matrix_type = MATRIX_TYPES
        .into_iter()
        .find(|matrix_type| {
            matrix_path
                .join(format!("{target}{}", matrix_type.file_suffix()))
                .is_file()
        })
        .ok_or_else(|| {
            anyhow::anyhow!("No clustermap matrix file was found for target {target}")
        })?;

    let sqm_path = matrix_path.join(format!("{target}{}", matrix_type.file_suffix()));
    let sqm = SquareMatrix::import_from_file(&sqm_path).with_context(|| {
        format!(
            "Failed to import Square Matrix data from '{}'",
            sqm_path.display()
        )
    })?;
    let matrix_name = matrix_type.display_name();

    plot_heat_phylo(sqm.clone(), cfg, target, matrix_name)
        .with_context(|| format!("Error plotting {target}-{matrix_name}_tree.svg"))?;
    // rename first plot for the purpose of the demo, since we are creating two
    // READ_PERCENTAGES plots and don't want the pie charts to get overridden
    fs::rename(
        cfg.io_args
            .output_path
            .join(format!("{target}-{matrix_name}.svg")),
        cfg.io_args
            .output_path
            .join(format!("{target}-{matrix_name}_tree.svg")),
    )
    .with_context(|| {
        format!("Error renaming {target}-{matrix_name}.svg to {target}-{matrix_name}_tree.svg")
    })?;

    plot_clustermap(sqm, cfg, target, matrix_name)
        .with_context(|| format!("Error plotting {target}-{matrix_name}.svg"))?;

    render_combined_demo_svg(&cfg.io_args.output_path, target, matrix_name)
        .with_context(|| "Error rendering combined demo SVG")?;

    Ok(())
}

/// Data and metadata for a single plot
#[derive(Debug)]
struct SvgPanel {
    pub width: f64,
    pub height: f64,
    // holds attributes from the original svgs from kuva, namely fonts
    pub root_attrs: String,
    pub body: String,
}

/// loads all of the rendered svgs and stitches them together
fn render_combined_demo_svg(output_path: &Path, target: &str, matrix_name: &str) -> Result<()> {
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
        .with_context(|| "Failed to write combined.svg")?;

    Ok(())
}

/// Reads in an svg's data and metadata
fn load_svg_panel(path: PathBuf, id_prefix: &str) -> Result<SvgPanel> {
    let svg = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read '{}'", path.display()))?;
    let open_start = svg
        .find("<svg")
        .with_context(|| format!("No <svg> tag found in '{}'", path.display()))?;
    let open_end = svg[open_start..]
        .find('>')
        .map(|idx| open_start + idx)
        .with_context(|| format!("Malformed <svg> tag in '{}'", path.display()))?;
    let root_tag = &svg[open_start..=open_end];
    let close_start = svg
        .rfind("</svg>")
        .with_context(|| format!("No closing </svg> tag found in '{}'", path.display()))?;

    let width = parse_svg_dimension(root_tag, "width")
        .with_context(|| format!("Missing width in '{}'", path.display()))?;
    let height = parse_svg_dimension(root_tag, "height")
        .with_context(|| format!("Missing height in '{}'", path.display()))?;
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

// gets metadata from a given svg
fn parse_svg_dimension(svg_tag: &str, attribute: &str) -> Result<f64> {
    let marker = format!("{attribute}=\"");
    let start = svg_tag
        .find(&marker)
        .map(|idx| idx + marker.len())
        .with_context(|| format!("Attribute {attribute:?} not found"))?;
    let end = svg_tag[start..]
        .find('"')
        .map(|idx| start + idx)
        .with_context(|| format!("Attribute {attribute:?} is not closed"))?;
    let raw = &svg_tag[start..end];
    let numeric = raw.trim_end_matches(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-')));

    numeric
        .parse::<f64>()
        .with_context(|| format!("Invalid {attribute:?} value {raw:?}"))
}

// get metadata (font) from an svg
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

fn extract_svg_attr(svg_tag: &str, attribute: &str) -> Option<String> {
    let marker = format!("{attribute}=\"");
    let start = svg_tag.find(&marker)? + marker.len();
    let end = svg_tag[start..].find('"')? + start;
    Some(svg_tag[start..end].to_owned())
}
