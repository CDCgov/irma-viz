# IRMA-Viz Changelog

All notable changes to this project will be documented in this file. The format
is roughly based on [Keep a Changelog], and this project tries to adheres to
[Semantic Versioning].

## [0.2.0-dev] - TBD

### Changed

- Changed default output format to `.pdf`, this can be changed in `config.toml`
- Histogam plots in `{ctype}-heuristics` now uses R-style pretty breaks as
  its binning strategy.
- Moved heuristics thresholds from `config.toml`: `min_f`, `min_tcc`, `min_aq`,
  `min_conf` are now CLI arguments only with range validation
- Made `variant_color`, `viz_option`, and `cluster_option` TOML arguments only
- Made `--paired` option for read-percentages CLI-only. It is required with no
  default
- Moved `tree height` to `[cluster_options]` in `config.toml`
- Enabled all clustermap matrix types by default in `config.toml`: `expenrd`,
  `jaccard`, `mutuald`, and `njointp`
- Made `--paired` CLI argument only required if `READ_PERCENTAGES` is toggled on
  and `viz_option = "pie"` is set
- Clarified CLI argument names for heuristics thresholds
  - `--min-aq` > `min-variant-average-quality`
  - `--min-f` > `--min-variant-frequency`
  - `--min-tcc` > `--min-variant-depth`
  - `--min-conf` > `--min-confidence-not-sequencer-error`
- Improved error and warning output to provide context for failed plots
- Made most plotting, data, and IO errors fallible, so that even if errors
  exist, as many plots as possible will still be created, and warnings will be
  provided via `std_err`

### Added

- Added pdf rendering capability
- Added ability to toggle all heuristics plots

## [0.1.1] - 2026-07-16

### Added

- Update CI pipeline to use Docker

## [0.1.0] - 2026-07-14

- Initial release. IRMA-Viz can reproduce the original IRMA plots.

<!-- Versions -->
[0.2.0-dev]: https://github.com/CDCgov/irma-viz/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/CDCgov/irma-viz/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/CDCgov/irma-viz/releases/tag/v0.1.0

<!-- Links -->
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
