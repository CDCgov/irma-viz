# irma-viz

`irma-viz` is a Rust CLI for rendering IRMA report plots as SVG files.

It currently reads tabular IRMA outputs plus `EXPENRD.sqm` matrices and renders:

- `READ_PERCENTAGES.svg` as either a sankey diagram or a pie-panel figure
- `{target}-heuristics.svg`
- `{target}-coverageDiagram.svg`
- `{target}-EXPENRD.svg` when variant data has more than one row

## Status

The project builds and runs, but it is still early-stage:

- the checked-in sample data is suitable for local smoke testing

## Build

```bash
cargo build
```

## Run

The binary loads a TOML config, then applies any CLI overrides on top.

```bash
cargo run -- --config path/to/config.toml
```

To write outputs somewhere else:

```bash
cargo run -- --config path/to/config.toml --output out/
```

To render only one target:

```bash
cargo run -- --config path/to/config.toml --targets A_NS --output out/
```

To override heuristic thresholds from the command line:

```bash
cargo run -- --config path/to/config.toml --min-aq 24 --min-f 0.008 --min-tcc 100 --min-conf 0.80
```

To override plot-specific options from the command line:

```bash
cargo run -- --config path/to/config.toml --coverage-variant-color frequency --read-percentages-viz pie --paired true --cluster-option tree
```

To override clustering layout height:

```bash
cargo run -- --config path/to/config.toml --tree-height 0.78
```

## CLI

Current help output:

```text
Render IRMA plots to SVG

Usage: irma-viz [OPTIONS]

Options:
      --config <CONFIG>                                  Path to config TOML [default: config.toml]
  -t, --targets <TARGETS>                                Target organisms to plot
      --table-path <TABLE_PATH>                          Path where the input data tables are held
      --matrix-path <MATRIX_PATH>                        Path where the input matrices are held
      --output <OUTPUT>                                  Output SVG path override
      --read-percentages <READ_PERCENTAGES>              [possible values: true, false]
      --heuristics <HEURISTICS>                          [possible values: true, false]
      --coverage <COVERAGE>                              [possible values: true, false]
      --clustermap <CLUSTERMAP>                          [possible values: true, false]
      --min-aq <MIN_AQ>
      --min-f <MIN_F>
      --min-tcc <MIN_TCC>
      --min-conf <MIN_CONF>
      --tree-height <TREE_HEIGHT>
      --coverage-variant-color <COVERAGE_VARIANT_COLOR>  [possible values: nucleotide, frequency]
      --read-percentages-viz <READ_PERCENTAGES_VIZ>      [possible values: sankey, pie]
      --paired <PAIRED>                                  [possible values: true, false]
      --cluster-option <CLUSTER_OPTION>                  [possible values: clustermap, tree]
  -h, --help                                             Print help
  -V, --version                                          Print version
```

## Config

The config file must match the current `Config` schema in `src/config.rs`

Valid top-level sections are:

- `targets`
- `plot_toggles`
- `input`
- `output`
- `constants`
- `coverage_options`
- `percent_options`
- `cluster_options`

Example:

```toml
[targets]
list = ["A_HA_H3", "A_MP", "A_NA_N2", "A_NP", "A_NS", "A_PA", "A_PB1", "A_PB2"]

[plot_toggles]
read_percentages = true
clustermap = true
heuristics = true
coverage = true

[input]
table_path = "test_tables/"
matrix_path = "test_matrices/"

[output]
path = "out/"

[constants]
min_aq = 24
min_f = 0.008
min_tcc = 100
min_conf = 0.80
tree_height = 0.78

[coverage_options]
variant_color = "nucleotide"

[percent_options]
viz_option = "pie"
paired = true

[cluster_options]
cluster_option = "clustermap"
matrix_types = { expenrd = true, jaccard = false, mutuald = false, njointp = false }
```

## Input Files

The program expects these files under `table_path`:

- `READ_COUNTS.txt`
- `{target}-allAlleles.txt`
- `{target}-coverage.txt`
- `{target}-pairingStats.txt`
- `{target}-variants.txt`

It expects these files under `matrix_path`:

- `{target}-EXPENRD.sqm`

## Output Files

Outputs are written as SVG files under `output.path` unless `--output` overrides it.

For each enabled target, the current renderer writes:

- `{target}-heuristics.svg`
- `{target}-coverageDiagram.svg`
- `{target}-EXPENRD.svg` when clustermap plotting is enabled and there is more than one variant row

It also writes:

- `READ_PERCENTAGES.svg` when `read_percentages` is enabled
  - rendered as a sankey when `percent_options.viz_option = "sankey"` or `--read-percentages-viz sankey`
  - rendered as a pie-panel figure when `percent_options.viz_option = "pie"` or `--read-percentages-viz pie`

## Notes

- CLI flags are overrides, not replacements for the config model.
- `--paired` only affects `READ_PERCENTAGES.svg`.
- `tree_height`, `cluster_option`, and `cluster_options.matrix_types` control which matrix-based cluster figures are rendered.
- The repository also includes `original_r_plots/` for reference outputs and `test_tables/` plus `test_matrices/` for local testing.
