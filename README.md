# IRMA-Viz

**As a first step, this document is under governance review. When the review
completes as appropriate per local and agency processes, the project team will
be allowed to remove this notice. This material is draft.**

## Overview

`irma-viz` is a Rust command-line tool for rendering IRMA (Iterative Refinement
Meta-Assembler) report plots as SVG files. The tool automates the visualization
of bioinformatic analysis results, converting tabular IRMA outputs and
quantitative matrices into useful figures.

**Purpose:** To provide fast, reliable plotting for IRMA reports, enabling
streamlined analysis workflows.

**Goals:**

- Reproduce the original IRMA visualization outputs faithfully
- Provide flexible configuration and command-line options for customization
- Support multiple plot types (sankey diagrams, pie charts, heatmaps, coverage
  diagrams)
- Maintain ease of use through config files and sensible defaults

## Features

`irma-viz` reads an IRMA output directory containing `tables/` and `matrices/`
subdirectories, discovers plot targets from the files that are present, and
renders:

- `READ_PERCENTAGES.svg` as either a sankey diagram or a pie-panel figure
- `{target}-heuristics.svg`
- `{target}-coverageDiagram.svg`
- `{target}-{matrix-type}.svg` for each enabled clustermap matrix type when
  matching matrix and variants inputs are present and the variants file has more
  than one variant

## Build

```bash
cargo build
```

## Run

The binary loads a TOML config, then applies CLI overrides on top. Input and
output paths are supplied on the command line, not in the config file.

```bash
cargo run -- --input-root path/to/irma-run
```

To use a non-default config:

```bash
cargo run -- --config path/to/config.toml --input-root path/to/irma-run
```

To write outputs somewhere else:

```bash
cargo run -- --input-root path/to/irma-run --output-path out/
```

To override heuristic thresholds from the command line:

```bash
cargo run -- --input-root path/to/irma-run --min-aq 24 --min-f 0.008 --min-tcc 100 --min-conf 0.80
```

To override plot-specific options from the command line:

```bash
cargo run -- --input-root path/to/irma-run --coverage-variant-color frequency --read-percentages-viz pie --paired true --cluster-option tree
```

To enable or disable plot types from the command line:

```bash
cargo run -- --input-root path/to/irma-run --coverage false --clustermap true
```

## CLI

Current help output:

```text
Render IRMA plots to SVG

Usage: irma-viz [OPTIONS] --input-root <INPUT_ROOT>

Options:
  -i, --input-root <INPUT_ROOT>
          Path to input directory that contains `tables/` and `matrices/`
  -o, --output-path <OUTPUT_PATH>
          Destination directory for output figures. If not specified, defaults to `input_root/figures/`
      --config <CONFIG>
          Path to config TOML [default: config.toml]
      --read-percentages <READ_PERCENTAGES>
          [possible values: true, false]
      --heuristics <HEURISTICS>
          [possible values: true, false]
      --coverage <COVERAGE>
          [possible values: true, false]
      --clustermap <CLUSTERMAP>
          [possible values: true, false]
      --min-aq <MIN_AQ>
      --min-f <MIN_F>
      --min-tcc <MIN_TCC>
      --min-conf <MIN_CONF>
      --tree-height <TREE_HEIGHT>
      --coverage-variant-color <COVERAGE_VARIANT_COLOR>
          [possible values: nucleotide, frequency]
      --read-percentages-viz <READ_PERCENTAGES_VIZ>
          [possible values: sankey, pie]
      --paired <PAIRED>
          [possible values: true, false]
      --cluster-option <CLUSTER_OPTION>
          [possible values: clustermap, tree]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Config

The config file must match the current `Config` schema in `src/config.rs`.
Unknown top-level sections are rejected. Input, output, and target selection are
not config-file fields in the current code.

Valid top-level sections are:

- `plot_toggles`
- `constants`
- `coverage_options`
- `percent_options`
- `cluster_options`

Example:

```toml
[plot_toggles]
read_percentages = true
clustermap = true
heuristics = true
coverage = true

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

`--input-root` points to an IRMA run directory. The code derives:

- `tables/` as `input_root/tables`
- `matrices/` as `input_root/matrices`

When read-percentages plotting is enabled, the program looks for this file
under `tables/`:

- `READ_COUNTS.txt`

If `READ_COUNTS.txt` is missing while read-percentages plotting is enabled, the
program prints a warning and continues.

Target-specific plots are discovered from files that are present:

- Heuristics targets come from `tables/{target}-allAlleles.txt`.
- Coverage targets come from any of these table files, but all three are
  required before coverage is plotted:
  - `{target}-variants.txt`
  - `{target}-coverage.txt`
  - `{target}-pairingStats.txt`
- Clustermap targets are discovered separately for each enabled matrix type in
  `cluster_options.matrix_types`. For each enabled type, targets come from the
  matching matrix file under `matrices/`; a matching
  `tables/{target}-variants.txt` file is also required.

Discovered target names must be non-empty, no more than 128 characters, and
contain only ASCII letters, numbers, `_`, `-`, or `.`. Files that imply other
target names are skipped with a warning.

Matrix files are optional per target and per enabled matrix type. IRMA only
creates clustermap matrices when the target has more than one variant, so the
absence of a matrix file means that target is not considered a clustermap target
for that matrix type. This is expected behavior, and `irma-viz` skips clustermap
plotting for that target and matrix type. If a matrix file is present but the
variants file has zero or one variant row, clustermap plotting is also skipped
for that target.

The supported clustermap matrix files are controlled by
`cluster_options.matrix_types`:

- `{target}-EXPENRD.sqm`
- `{target}-JACCARD.sqm`
- `{target}-MUTUALD.sqm`
- `{target}-NJOINTP.sqm`

When multiple matrix types are enabled, `irma-viz` checks each type
independently. If a valid clustermap target is found for at least one enabled
matrix type but not another, the program prints a warning and renders the plots
for the matrix files that are present.

Square matrix files are read as tab-delimited rows where the first field is the
row label and the remaining fields are floating-point values. Blank lines are
ignored. The matrix import fails if the file is empty, any value cannot be
parsed as a float, rows have different lengths, or the number of rows does not
match the number of numeric columns.

The target-specific filenames are:

- `{target}-allAlleles.txt`
- `{target}-coverage.txt`
- `{target}-pairingStats.txt`
- `{target}-variants.txt`
- `{target}-EXPENRD.sqm`
- `{target}-JACCARD.sqm`
- `{target}-MUTUALD.sqm`
- `{target}-NJOINTP.sqm`

## Output Files

Outputs are written as SVG files under `input_root/figures/` unless
`--output-path` overrides the destination directory. The output directory is
created if needed.

For each discovered target, the renderer writes the following files based on
enabled plot toggles and available inputs:

- `{target}-heuristics.svg` when `plot_toggles.heuristics = true`
- `{target}-coverageDiagram.svg` when `plot_toggles.coverage = true`
- `{target}-{matrix-type}.svg` when `plot_toggles.clustermap = true`, the
  matrix type is enabled in `cluster_options.matrix_types`, a matching
  `{target}-{matrix-type}.sqm` matrix exists, and the variant data has more
  than one row
  - rendered as a static clustermap when `cluster_options.cluster_option =
    "clustermap"`
  - rendered as a phylogenetic tree with heatmap when
    `cluster_options.cluster_option = "tree"`

Additionally, the renderer writes:

- `READ_PERCENTAGES.svg` when `plot_toggles.read_percentages = true`
  - rendered as a sankey diagram when `percent_options.viz_option = "sankey"` or
    `--read-percentages-viz sankey`
  - rendered as a pie-panel figure when `percent_options.viz_option = "pie"` or
    `--read-percentages-viz pie`

## Notes

- CLI flags are overrides, not replacements for the config model.
- `--paired` only affects `READ_PERCENTAGES.svg`.
- `cluster_option` affects `{target}-{matrix-type}.svg` clustermap outputs.
- `tree_height` and `--tree-height` affect tree-style
  `{target}-{matrix-type}.svg` plots.

## Notices

### Contact Info

For direct correspondence on the project, feel free to contact: [Samuel S.
Shepard](mailto:sshepard@cdc.gov), Centers for Disease Control and Prevention or
reach out to other [contributors](CONTRIBUTORS.md).

### Public Domain Standard Notice

This repository constitutes a work of the United States Government and is not
subject to domestic copyright protection under 17 USC § 105. This repository is
in the public domain within the United States, and copyright and related rights
in the work worldwide are waived through the [CC0 1.0 Universal public domain
dedication](https://creativecommons.org/publicdomain/zero/1.0/). All
contributions to this repository will be released under the CC0 dedication. By
submitting a pull request you are agreeing to comply with this waiver of
copyright interest.

### License Standard Notice

The repository utilizes code licensed under the terms of the Apache Software
License and therefore is licensed under ASL v2 or later. This source code in
this repository is free: you can redistribute it and/or modify it under the
terms of the Apache Software License version 2, or (at your option) any later
version. This source code in this repository is distributed in the hope that it
will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the Apache Software
License for more details. You should have received a copy of the Apache Software
License along with this program. If not, see:
<http://www.apache.org/licenses/LICENSE-2.0.html>. The source code forked from
other open source projects will inherit its license.

### Privacy Standard Notice

This repository contains only non-sensitive, publicly available data and
information. All material and community participation is covered by the
[Disclaimer](https://github.com/CDCgov/template/blob/main/DISCLAIMER.md). For
more information about CDC's privacy policy, please visit
<http://www.cdc.gov/other/privacy.html>.

### Contributing Standard Notice

Anyone is encouraged to contribute to the repository by
[forking](https://help.github.com/articles/fork-a-repo) and submitting a pull
request. (If you are new to GitHub, you might start with a [basic
tutorial](https://help.github.com/articles/set-up-git).) By contributing to this
project, you grant a world-wide, royalty-free, perpetual, irrevocable,
non-exclusive, transferable license to all users under the terms of the [Apache
Software License v2](http://www.apache.org/licenses/LICENSE-2.0.html) or later.

All comments, messages, pull requests, and other submissions received through
CDC including this GitHub page may be subject to applicable federal law,
including but not limited to the Federal Records Act, and may be archived. Learn
more at
[http://www.cdc.gov/other/privacy.html](http://www.cdc.gov/other/privacy.html).

### Records Management Standard Notice

This repository is not a source of government records, but is a copy to increase
collaboration and collaborative potential. All government records will be
published through the [CDC web site](http://www.cdc.gov).

## Additional Standard Notices

Please refer to [CDC's Template Repository](https://github.com/CDCgov/template)
for more information about [contributing to this
repository](https://github.com/CDCgov/template/blob/main/CONTRIBUTING.md),
[public domain notices and
disclaimers](https://github.com/CDCgov/template/blob/main/DISCLAIMER.md), and
[code of
conduct](https://github.com/CDCgov/template/blob/main/code-of-conduct.md).
