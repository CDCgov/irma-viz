use crate::{
    config::Config,
    data::{ReadCounts, SankeyVec},
    plots::{render_multiplot, render_plot},
};
use anyhow::{Result, anyhow};
use kuva::{
    Palette,
    plot::{LegendEntry, LegendShape, PiePlot, SankeyPlot, TextPlot},
    prelude::{Figure, Layout, Plot},
};

pub fn plot_perc_sankey(sankey_vec: SankeyVec, cfg: &Config) -> Result<()> {
    let (plot, layout) = kuva_sankey(sankey_vec);

    render_plot(
        ("READ_PERCENTAGES.svg", (plot, layout)),
        cfg.output.path.clone(),
    )
}

fn kuva_sankey(sankey_vec: SankeyVec) -> (Vec<Plot>, Layout) {
    let sankey = SankeyPlot::new()
        .with_links(sankey_vec.edges)
        .with_flow_labels()
        .with_flow_label_min_height(0.0);
    let plots = vec![sankey.into()];
    let layout = Layout::auto_from_plots(&plots);
    (plots, layout)
}

pub fn plot_perc_pies(read_counts: ReadCounts, cfg: &Config) -> Result<()> {
    let paired = cfg.plot_specific.read_percent.paired;

    let pal = Palette::wong();

    // --------------------- Totals Pie Chart -----------------------
    let mut vals = Vec::with_capacity(3);
    let mut legend_labels = Vec::with_capacity(3);

    let total = read_counts.read("1-initial");
    let nonqual = read_counts.read("2-failQC");
    let assembled = read_counts.read("3-match");
    let therest = total - nonqual - assembled;

    if assembled > 0.0 {
        vals.push(assembled);
        legend_labels.push("Assembled");
    }
    if nonqual > 0.0 {
        vals.push(nonqual);
        legend_labels.push("QC filtered");
    }
    if therest > 0.0 {
        vals.push(therest);
        legend_labels.push("Other");
    }

    if vals.is_empty() {
        return Err(anyhow!("Empty Totals pie chart in READ_PERCENTAGES"));
    }
    let (legend_entries, total_pie) = kuva_pie(vals, &legend_labels, &pal);

    let total_pie = vec![total_pie.with_legend("").with_percent().into()];
    let total_layout = Layout::auto_from_plots(&total_pie)
        .with_title({
            if paired {
                "1. Percentages of total reads (R1 + R2)"
            } else {
                "1. Percentages of total reads"
            }
        })
        .with_legend_entries(legend_entries)
        .with_legend_position(kuva::plot::LegendPosition::OutsideRightBottom);

    // ------------------- Passing QC -----------------------
    let mut vals = Vec::with_capacity(4);
    let mut legend_labels = Vec::with_capacity(4);

    let assembled = read_counts.pattern("3-match");
    let chimeric = read_counts.pattern("3-chimeric");
    let unused = read_counts.pattern("3-unrecognizable") + read_counts.pattern("3-altmatch");
    let unmatched = read_counts.pattern("3-nomatch");

    if assembled > 0.0 {
        vals.push(assembled);
        legend_labels.push("Assembled");
    }
    if unused > 0.0 {
        vals.push(unused);
        legend_labels.push("Unusable");
    }
    if chimeric > 0.0 {
        vals.push(chimeric);
        legend_labels.push("Chimeric");
    }
    if unmatched > 0.0 {
        vals.push(unmatched);
        legend_labels.push("No match");
    }

    if vals.is_empty() {
        return Err(anyhow!("Empty Pass QC pie chart in READ_PERCENTAGES"));
    }
    let (legend_entries, passed_qc_pie) = kuva_pie(vals, &legend_labels, &pal);

    let passed_qc_pie = vec![
        passed_qc_pie
            .with_legend("")
            .with_percent()
            .with_label_position(kuva::plot::PieLabelPosition::Outside)
            .into(),
    ];
    let passed_qc_layout = Layout::auto_from_plots(&passed_qc_pie)
        .with_title("2. Percentages of all read patterns passing QC")
        .with_legend_entries(legend_entries)
        .with_legend_position(kuva::plot::LegendPosition::OutsideRightTop);

    // -------------------- Matches --------------------------------------------

    let (targets, mut vals): (Vec<_>, Vec<_>) = {
        let mut pairs = read_counts
            .map
            .keys()
            .filter_map(|key| {
                key.strip_prefix("4-")
                    .map(|record| (record, read_counts.pairs_and_widows(key)))
            })
            .collect::<Vec<_>>();

        pairs.sort_unstable_by(|a, b| a.0.cmp(b.0));
        pairs.into_iter().unzip()
    };
    vals.reverse();
    let total = vals.iter().sum::<f64>();
    if total == 0.0 {
        return Err(anyhow!("Empty Matches pie chart in READ_PERCENTAGES"));
    }
    let mut slice_labels = Vec::with_capacity(vals.len());
    for (&val, target) in vals.iter().zip(targets) {
        if val / total < 0.02 {
            slice_labels.push(target.to_string())
        } else if val >= 1_000_000.0 {
            slice_labels.push(format!("{target}: {:.1}M", val / 1_000_000.0))
        } else if val >= 1_000.0 {
            slice_labels.push(format!("{target}: {:.1}k", val / 1_000.0))
        } else {
            slice_labels.push(format!("{target}: {val}"))
        }
    }

    let mut match_pie = PiePlot::new();
    for (idx, (&val, slice_label)) in vals.iter().zip(slice_labels).enumerate() {
        match_pie = match_pie.with_slice(slice_label, val, &pal[vals.len() - 1 - idx]);
    }

    let match_pie = vec![
        match_pie
            //.with_legend("")
            .with_percent()
            .with_label_position(kuva::plot::PieLabelPosition::Outside)
            .into(),
    ];
    let match_layout = Layout::auto_from_plots(&match_pie)
        .with_title({
            if paired {
                "3. Percentages of assembled, merged-pair reads"
            } else {
                "3. Percentages of assembled reads"
            }
        })
        .with_title_wrap(28);

    // --------------------- Text Box -----------------
    let text_box = TextPlot::new()
        .with_body(if paired { PAIRED_README } else { SINGLE_README })
        .with_padding(0.0);
    let text_box = vec![text_box.into()];
    let text_box_layout = Layout::auto_from_plots(&text_box);

    // ---------------------- Multi-Plot ----------------------------
    let plots = vec![total_pie, passed_qc_pie, match_pie, text_box];
    let layouts = vec![
        total_layout,
        passed_qc_layout,
        match_layout,
        text_box_layout,
    ];

    let filename = "READ_PERCENTAGES.svg";

    let scene = Figure::new(2, 2)
        .with_plots(plots)
        .with_layouts(layouts)
        .render();

    render_multiplot(&scene, cfg.output.path.clone(), filename)
}

fn kuva_pie(
    mut vals: Vec<f64>,
    legend_labels: &[&str],
    pal: &Palette,
) -> (Vec<LegendEntry>, PiePlot) {
    vals.reverse();
    let slice_labels = make_slice_labels(&vals);
    let mut legend_entries = Vec::with_capacity(legend_labels.len());
    let mut pie = PiePlot::new();
    for (idx, ((&val, slice_label), &legend_label)) in
        vals.iter().zip(slice_labels).zip(legend_labels).enumerate()
    {
        pie = pie.with_slice(slice_label, val, &pal[vals.len() - 1 - idx]);
        legend_entries.push(LegendEntry {
            label: legend_label.into(),
            color: pal[idx].to_string(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
    }
    (legend_entries, pie)
}

fn make_slice_labels(vals: &[f64]) -> Vec<String> {
    let mut slice_labels = Vec::with_capacity(vals.len());
    for &val in vals {
        if val >= 1_000_000.0 {
            slice_labels.push(format!("{:.1}M", val / 1_000_000.0))
        } else if val >= 1_000.0 {
            slice_labels.push(format!("{:.1}k", val / 1_000.0))
        } else {
            slice_labels.push(format!("{val}"))
        }
    }
    slice_labels
}

const SINGLE_README: &str = "# READ PROPORTIONS.\n\
\n\
## 1. Percentages of total read counts\n\
    - ASSEMBLED: influenza reads in final assemblies.\n\
    - QC FILTERED: didn't pass length/median quality thresholds.\n\
    - OTHER: non-flu and contaminant/poor flu signal.\n\
\n\
## 2. Percentages of all read patterns passing QC process\n\
   - Patterns are clustered or non-redundant reads.\n\
   - ASSEMBLED: excellent influenza read patterns.\n\
   - UNUSABLE: poor or contaminant flu patterns.\n\
   - CHIMERIC: flu patterns matching both strands.\n\
   - NO MATCH: non-flu read patterns.\n\
\n\
## 3. Percentages of assembled read counts\n\
   - Shows the proportion of gene segments to the genome.";

const PAIRED_README: &str = "# READ PROPORTIONS.\n\
\n\
## 1. Percentages of total read counts (R1 & R2)\n\
    - **ASSEMBLED**: influenza reads in final assemblies.\n\
    - **QC FILTERED**: didn't pass length/median quality thresholds.\n\
    - **OTHER**: non-flu and contaminant/poor flu signal.\n\
\n\
## 2. Percentages of all read patterns passing QC process\n\
   - Patterns are clustered or non-redundant reads.\n\
   - **ASSEMBLED**: excellent influenza read patterns.\n\
   - **UNUSABLE**: poor or contaminant flu patterns.\n\
   - **CHIMERIC**: flu patterns matching both strands.\n\
   - **NO MATCH**: non-flu read patterns.\n\
\n\
## 3. Percentages of assembled, merged-pair read counts\n\
   - Shows the proportion of gene segments to the genome.\n\
   - Paired-end reads have been merged into a single count\n\
     unless not applicable: single-end reads have been used.";
