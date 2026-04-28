use crate::{
    config::{Config, PercentVizOption},
    data::SankeyVec,
    plots::{render_multiplot, render_plot},
};
use anyhow::Result;
use kuva::{
    plot::{PiePlot, SankeyPlot},
    prelude::{Figure, Layout, Plot},
    render::annotations::TextAnnotation,
};
use std::collections::HashMap;

pub fn plot_read_percentages(sankey_vec: SankeyVec, cfg: &Config) -> Result<()> {
    if cfg.plot_specific.read_percent.viz_option == PercentVizOption::Sankey {
        let (plot, layout) = kuva_sankey(sankey_vec);

        render_plot(
            ("READ_PERCENTAGES.svg", (plot, layout)),
            cfg.output.path.clone(),
        )
    } else {
        total_reads_pies(sankey_vec, cfg)
    }
}

pub fn kuva_sankey(sankey_vec: SankeyVec) -> (Vec<Plot>, Layout) {
    let sankey = SankeyPlot::new()
        .with_links(sankey_vec.edges)
        .with_flow_labels()
        .with_flow_label_min_height(0.0);
    let plots = vec![sankey.into()];
    let layout = Layout::auto_from_plots(&plots);
    (plots, layout)
}

fn total_reads_pies(sankey_vec: SankeyVec, cfg: &Config) -> Result<()> {
    let paired = true;

    let mut map = HashMap::new();
    for (from, to, value) in &sankey_vec.edges {
        map.insert((from.as_str(), to.as_str()), *value);
    }

    let pass_qc = *map.get(&("Initial Reads", "Pass QC")).unwrap_or(&0.0);
    let fail_qc = *map.get(&("Initial Reads", "Fail QC")).unwrap_or(&0.0);
    let total = pass_qc + fail_qc;

    let pass_pct = if total > 0.0 {
        100.0 * pass_qc / total
    } else {
        0.0
    };
    let fail_pct = if total > 0.0 {
        100.0 * fail_qc / total
    } else {
        0.0
    };
    // TO-DO - replace placeholder colors
    let total_pie = PiePlot::new()
        .with_slice("Pass QC", pass_pct, "seagreen")
        .with_slice("Fail QC", fail_pct, "tomato")
        .with_legend("QC result")
        .with_percent()
        .with_label_position(kuva::plot::PieLabelPosition::Outside);
    let total_pie = vec![total_pie.into()];
    let total_layout = Layout::auto_from_plots(&total_pie).with_title({
        if paired {
            "1. Percentages of total reads (R1 + R2)"
        } else {
            "1. Percentages of total reads"
        }
    });

    let primary = *map.get(&("Pass QC", "Primary Match")).unwrap_or(&0.0);
    let alt = *map.get(&("Pass QC", "Alt Match")).unwrap_or(&0.0);
    let no_match = *map.get(&("Pass QC", "No Match")).unwrap_or(&0.0);
    let chimeric = *map.get(&("Pass QC", "Chimeric")).unwrap_or(&0.0);

    let total = primary + alt + no_match + chimeric;

    let primary_pct = if total > 0.0 {
        100.0 * primary / total
    } else {
        0.0
    };
    let alt_pct = if total > 0.0 {
        100.0 * alt / total
    } else {
        0.0
    };
    let no_match_pct = if total > 0.0 {
        100.0 * no_match / total
    } else {
        0.0
    };
    let chimeric_pct = if total > 0.0 {
        100.0 * chimeric / total
    } else {
        0.0
    };
    // TO-DO - replace placeholder colors
    let passed_qc_pie = PiePlot::new()
        .with_slice("Primary Match", primary_pct, "steelblue")
        .with_slice("Alt Match", alt_pct, "orange")
        .with_slice("No Match", no_match_pct, "gray")
        .with_slice("Chimeric", chimeric_pct, "purple")
        .with_percent()
        .with_label_position(kuva::plot::PieLabelPosition::Outside)
        .with_legend("Match Type");
    let passed_qc_pie = vec![passed_qc_pie.into()];
    let passed_qc_layout = Layout::auto_from_plots(&passed_qc_pie)
        .with_title("2. Percentages of all read patterns passing QC");

    let primary_matches: Vec<(&str, f64)> = sankey_vec
        .edges
        .iter()
        .filter_map(|(from, to, value)| {
            if from == "Primary Match" {
                Some((to.as_str(), *value))
            } else {
                None
            }
        })
        .collect();

    let total: f64 = primary_matches.iter().map(|(_, v)| *v).sum();
    let mut match_pie = PiePlot::new()
        .with_percent()
        .with_label_position(kuva::plot::PieLabelPosition::Outside);
    //.with_legend("Primary Classification")

    for (label, value) in primary_matches {
        let pct = if total > 0.0 {
            100.0 * value / total
        } else {
            0.0
        };

        // TO-DO - replace placeholder colors
        let color = match label {
            "A_MP" => "steelblue",
            "A_NP" => "seagreen",
            "A_PB2" => "orange",
            "A_HA_H3" => "tomato",
            "A_PA" => "purple",
            "A_PB1" => "gold",
            "A_NS" => "brown",
            "A_NA_N2" => "gray",
            _ => "lightgray",
        };

        match_pie = match_pie.with_slice(label, pct, color);
    }
    let match_pie = vec![match_pie.into()];
    let match_layout = Layout::auto_from_plots(&match_pie).with_title({
        if paired {
            "3. Percentages of assembled, merged-pair reads"
        } else {
            "3. Percentages of assembled reads"
        }
    });

    // Placeholder for paragraph text
    // this is ugly as sin but should have something more functional in the kuva 0.1.17 release
    let text_box = PiePlot::new()
        .with_slice("test", 100.0, "purple")
        .with_legend("hey");
    let text_box = vec![text_box.into()];

    let text_box_layout = Layout::auto_from_plots(&text_box).with_annotation(
        TextAnnotation::new(if paired { PAIRED_README } else { SINGLE_README }, 0.0, 0.0)
            .with_font_size(12),
    );

    let filename = "READ_PERCENTAGES.svg";

    let scene = Figure::new(2, 2)
        .with_plots(vec![total_pie, passed_qc_pie, match_pie, text_box])
        .with_layouts(vec![
            total_layout,
            passed_qc_layout,
            match_layout,
            text_box_layout,
        ])
        .render();

    render_multiplot(&scene, cfg.output.path.clone(), filename)
}

const SINGLE_README: &str = "READ PROPORTIONS.

1. Percentages of total read counts
    - ASSEMBLED: influenza reads in final assemblies.
    - QC FILTERED: didn't pass length/median quality thresholds.
    - OTHER: non-flu and contaminant/poor flu signal.

2. Percentages of all read patterns passing QC process
   - Patterns are clustered or non-redundant reads.
   - ASSEMBLED: excellent influenza read patterns.
   - UNUSABLE: poor or contaminant flu patterns.
   - CHIMERIC: flu patterns matching both strands.
   - NO MATCH: non-flu read patterns.

3. Percentages of assembled read counts
   - Shows the proportion of gene segments to the genome.";

const PAIRED_README: &str = "READ PROPORTIONS.

1. Percentages of total read counts (R1 & R2)
    - ASSEMBLED: influenza reads in final assemblies.
    - QC FILTERED: didn't pass length/median quality thresholds.
    - OTHER: non-flu and contaminant/poor flu signal.

2. Percentages of all read patterns passing QC process
   - Patterns are clustered or non-redundant reads.
   - ASSEMBLED: excellent influenza read patterns.
   - UNUSABLE: poor or contaminant flu patterns.
   - CHIMERIC: flu patterns matching both strands.
   - NO MATCH: non-flu read patterns.

3. Percentages of assembled, merged-pair read counts
   - Shows the proportion of gene segments to the genome.
   - Paired-end reads have been merged into a single count
     unless not applicable: single-end reads have been used.";
