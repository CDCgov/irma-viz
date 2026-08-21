# irma-viz Changelog

All notable changes to this project will be documented in this file. The format
is roughly based on [Keep a Changelog], and this project tries to adhere to
[Semantic Versioning].

## [0.2.0-dev] - TBD

### Changed

- Changed the default output format to `.pdf`; it can be changed in
  `config.toml`
- Histogram plots in `{ctype}-heuristics` now use R-style pretty breaks for
  binning.
- Moved heuristics thresholds from `config.toml`: `min_f`, `min_tcc`, `min_aq`,
  `min_conf` are now CLI arguments only with range validation
- Made `variant_color`, `viz_option`, and `cluster_option` TOML options only
- Made `--paired` CLI-only, with no default; it is required only when
  `READ_PERCENTAGES` is enabled with `viz_option = "pie"`
- Moved `tree_height` to `[cluster_options]` in `config.toml`
- Enabled all clustermap matrix types by default in `config.toml`: `expenrd`,
  `jaccard`, `mutuald`, and `njointp`
- Clarified CLI argument names for heuristics thresholds
  - `--min-aq` > `--min-variant-average-quality`
  - `--min-f` > `--min-variant-frequency`
  - `--min-tcc` > `--min-variant-depth`
  - `--min-conf` > `--min-confidence-not-sequencer-error`
- Improved error and warning output to provide context for failed plots
- Made most plotting, data, and IO errors fallible, so that even if errors
  exist, as many plots as possible will still be created, and warnings will be
  provided via `std_err`

### Added

- Added PDF rendering capability
- Added the ability to toggle individual heuristics subplots

## [0.1.1] - 2026-07-16

### Added

- Update CI pipeline to use Docker

## [0.1.0] - 2026-07-14

- Initial release. irma-viz can reproduce the original IRMA plots.

<!-- Versions -->
[0.2.0-dev]: https://github.com/CDCgov/irma-viz/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/CDCgov/irma-viz/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/CDCgov/irma-viz/releases/tag/v0.1.0

<!-- Links -->
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
