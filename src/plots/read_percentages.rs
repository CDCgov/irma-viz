use crate::{
    config::Config,
    data::{ReadCounts, SankeyVec},
    plots::{render_multiplot, render_plot},
};
use anyhow::Result;
use kuva::{
    Palette, plot::{PiePlot, SankeyPlot, TextPlot}, prelude::{Figure, Layout, Plot}
};

pub fn plot_perc_sankey(sankey_vec: SankeyVec, cfg: &Config) -> Result<()> {
    let (plot, layout) = kuva_sankey(sankey_vec);

    render_plot(
        ("READ_PERCENTAGES.svg", (plot, layout)),
        cfg.output.path.clone(),
    )
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

pub fn plot_perc_pies(read_counts: ReadCounts, cfg: &Config) -> Result<()> {
    // TODO: Auto pick colors
    // TODO: pie chart values do not need to be based on magnitude, api is proportional

    let targets = &cfg.targets.list;
    let (plots, layouts) = kuva_pies(read_counts, targets);

    let filename = "READ_PERCENTAGES.svg";

    let scene = Figure::new(2, 2)
        .with_plots(plots)
        .with_layouts(layouts)
        .render();

    render_multiplot(&scene, cfg.output.path.clone(), filename)
}

fn kuva_pies(read_counts: ReadCounts, targets: &[String]) -> (Vec<Vec<Plot>>, Vec<Layout>) {
    let paired = true;

    let pal = Palette::wong();

    let map = read_counts.record_data_map;
    let pass_qc = *map.get("2-passQC").unwrap_or(&0.0);
    let fail_qc = *map.get("2-failQC").unwrap_or(&0.0);

    let total_pie = PiePlot::new()
        .with_slice("Pass QC", pass_qc, &pal[0])
        .with_slice("Fail QC", fail_qc, &pal[1])
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

    let primary = *map.get("3-match").unwrap_or(&0.0);
    let alt = *map.get("3-altmatch").unwrap_or(&0.0);
    let no_match = *map.get("3-nomatch").unwrap_or(&0.0);
    let chimeric = *map.get("3-chimeric").unwrap_or(&0.0);

    let passed_qc_pie = PiePlot::new()
        .with_slice("Primary Match", primary, &pal[0])
        .with_slice("Alt Match", alt, &pal[1])
        .with_slice("No Match", no_match, &pal[2])
        .with_slice("Chimeric", chimeric, &pal[3])
        .with_percent()
        .with_label_position(kuva::plot::PieLabelPosition::Outside)
        .with_legend("Match Type");
    let passed_qc_pie = vec![passed_qc_pie.into()];
    let passed_qc_layout = Layout::auto_from_plots(&passed_qc_pie)
        .with_title("2. Percentages of all read patterns passing QC");

    let primary_matches = targets
        .iter()
        .map(|target| {
            let mut prefix_target = String::from("4-");
            prefix_target.push_str(target);
            (target.as_str(), *map.get(&prefix_target).unwrap_or(&0.0))
        })
        .collect::<Vec<_>>();

    let mut match_pie = PiePlot::new()
        .with_percent()
        .with_label_position(kuva::plot::PieLabelPosition::Outside);
    //.with_legend("Primary Classification")

    for (index, (label, value)) in primary_matches.into_iter().enumerate() {
        let color = &pal[index];
        match_pie = match_pie.with_slice(label, value, color);
    }
    let match_pie = vec![match_pie.into()];
    let match_layout = Layout::auto_from_plots(&match_pie).with_title({
        if paired {
            "3. Percentages of assembled, merged-pair reads"
        } else {
            "3. Percentages of assembled reads"
        }
    });

    let text_box = TextPlot::new()
        .with_body(if paired { PAIRED_README } else { SINGLE_README })
        .with_padding(0.0);
    let text_box = vec![text_box.into()];
    let text_box_layout = Layout::auto_from_plots(&text_box);

    (
        vec![total_pie, passed_qc_pie, match_pie, text_box],
        vec![
            total_layout,
            passed_qc_layout,
            match_layout,
            text_box_layout,
        ],
    )
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
