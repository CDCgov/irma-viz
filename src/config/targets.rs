use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::config::{
    matrices::{MatrixType, MatrixTypes},
    parsed_config::{IOConfig, PlotToggles, get_directory_paths},
};

const HEURISTICS_REQUIRED_SUFFIXES: &[&str] = &["-allAlleles.txt"];
const COVERAGE_REQUIRED_TABLE_SUFFIXES: &[&str] =
    &["-variants.txt", "-coverage.txt", "-pairingStats.txt"];
const CLUSTERMAP_REQUIRED_TABLE_SUFFIXES: &[&str] = &["-variants.txt"];

/// Stores a list of targets seperately for each matrix type
#[derive(Debug, Default)]
pub struct ClusterTargets {
    pub expenrd: BTreeSet<String>,
    pub jaccard: BTreeSet<String>,
    pub mutuald: BTreeSet<String>,
    pub njointp: BTreeSet<String>,
}

impl ClusterTargets {
    fn insert(&mut self, matrix_type: MatrixType, target: String) {
        self.targets_for_mut(matrix_type).insert(target);
    }

    /// gets the set of targets for a given matrix type
    pub fn targets_for(&self, matrix_type: MatrixType) -> &BTreeSet<String> {
        match matrix_type {
            MatrixType::Expenrd => &self.expenrd,
            MatrixType::Jaccard => &self.jaccard,
            MatrixType::Mutuald => &self.mutuald,
            MatrixType::Njointp => &self.njointp,
        }
    }

    /// gets the set of targets for a given matrix type mutably
    fn targets_for_mut(&mut self, matrix_type: MatrixType) -> &mut BTreeSet<String> {
        match matrix_type {
            MatrixType::Expenrd => &mut self.expenrd,
            MatrixType::Jaccard => &mut self.jaccard,
            MatrixType::Mutuald => &mut self.mutuald,
            MatrixType::Njointp => &mut self.njointp,
        }
    }

    fn variant_targets(&self) -> BTreeSet<String> {
        self.expenrd
            .union(&self.jaccard)
            .chain(self.mutuald.union(&self.njointp))
            .cloned()
            .collect()
    }

    // a big ugly way to ensure that if a matrix file exists for a certain
    // matrix type, we warn if it doesn't exist for the other enabled matrix type(s)
    fn check_missing_matrix_targets(&self, matrix_types: &MatrixTypes) {
        let enabled = matrix_types.enabled_matrix_types();
        if enabled.len() < 2 {
            return;
        }

        let any_enabled_targets: BTreeSet<String> = enabled
            .iter()
            .flat_map(|matrix_type| self.targets_for(*matrix_type).iter().cloned())
            .collect();

        for target in any_enabled_targets {
            let present_in = enabled
                .iter()
                .filter(|matrix_type| self.targets_for(**matrix_type).contains(&target))
                .map(|matrix_type| matrix_type.display_name())
                .collect::<Vec<_>>();

            if present_in.len() == enabled.len() {
                continue;
            }

            let missing_from = enabled
                .iter()
                .filter(|matrix_type| !self.targets_for(**matrix_type).contains(&target))
                .map(|matrix_type| matrix_type.display_name())
                .collect::<Vec<_>>();

            eprintln!(
                "Warning: clustermap matrix files were found for target {} for enabled matrix {} but not in enabled {}",
                target,
                present_in.join(", "),
                missing_from.join(", ")
            );
        }
    }
}

/// A collection of confirmed targets, stored separately for each plot type
#[derive(Debug, Default)]
pub struct PlotTargets {
    pub heuristics: BTreeSet<String>,
    pub coverage: BTreeSet<String>,
    pub clustermap: ClusterTargets,
}

impl PlotTargets {
    pub fn variant_targets(&self) -> BTreeSet<String> {
        self.coverage
            .union(&self.clustermap.variant_targets())
            .cloned()
            .collect()
    }

    /// cross-check the targets for each plot type to see if there are targets
    /// missing from one plot but not another. Also checks if no targets were
    /// found for a given plot type (excluding clustermap)
    pub fn check_missing_targets(&self, plot_toggles: &PlotToggles, matrix_types: &MatrixTypes) {
        if plot_toggles.heuristics && self.heuristics.is_empty() {
            eprintln!("Warning: heuristics plotting was enabled but no valid targets were found");
        }
        if plot_toggles.coverage && self.coverage.is_empty() {
            eprintln!("Warning: coverage plotting was enabled but not valid targets were found");
        }

        if plot_toggles.heuristics && plot_toggles.coverage {
            warn_missing(&self.heuristics, &self.coverage, "heuristics", "coverage");
            warn_missing(&self.coverage, &self.heuristics, "coverage", "heuristics");
        }

        let clustermap_targets = self.clustermap.variant_targets();

        if plot_toggles.heuristics && plot_toggles.clustermap {
            warn_missing(
                &clustermap_targets,
                &self.heuristics,
                "clustermap",
                "heuristics",
            );
        }

        if plot_toggles.coverage && plot_toggles.clustermap {
            warn_missing(
                &clustermap_targets,
                &self.coverage,
                "clustermap",
                "coverage",
            );
        }

        if plot_toggles.clustermap {
            self.clustermap.check_missing_matrix_targets(matrix_types);
        }
    }
}

/// helper function for warning of missing files for a given target and plot type
fn warn_missing<'a>(
    from: &'a BTreeSet<String>,
    to: &'a BTreeSet<String>,
    from_name: &str,
    to_name: &str,
) {
    for target in from.difference(to) {
        eprintln!(
            "Warning: necessary files found to create {from_name} plot but not {to_name} plot for target {target}"
        );
    }
}

/// get path to `tables` and `matrices` directories before calling to
/// [discover_targets_by_plot_type]
pub fn resolve_targets(
    plot_toggles: &PlotToggles,
    io_args: &IOConfig,
    matrix_types: &MatrixTypes,
) -> Result<PlotTargets> {
    // no plots needing targets enabled
    if !(plot_toggles.heuristics || plot_toggles.coverage || plot_toggles.clustermap) {
        return Ok(PlotTargets::default());
    }

    let (table_path, matrix_path) = get_directory_paths(&io_args.input_root);
    discover_targets_by_plot_type(&table_path, &matrix_path, plot_toggles, matrix_types)
}

/// finds all valid targets for each given plot type and stores them seperately
fn discover_targets_by_plot_type(
    table_dir: &Path,
    matrix_dir: &Path,
    plot_toggles: &PlotToggles,
    matrix_types: &MatrixTypes,
) -> Result<PlotTargets> {
    let mut plot_targets = PlotTargets::default();
    // collects all possible heuristics targets
    if plot_toggles.heuristics {
        let possible_heuristics_targets =
            discover_candidate_targets(table_dir, HEURISTICS_REQUIRED_SUFFIXES)?;
        for possible_target in possible_heuristics_targets {
            let required_heuristics_files =
                required_target_files(table_dir, &possible_target, HEURISTICS_REQUIRED_SUFFIXES);
            if validate_target_files(&possible_target, required_heuristics_files, "heuristics") {
                plot_targets.heuristics.insert(possible_target);
            }
        }
    }

    // collects all possible coverage targets
    if plot_toggles.coverage {
        // all of the potential targets we see
        let possible_coverage_targets =
            discover_candidate_targets(table_dir, COVERAGE_REQUIRED_TABLE_SUFFIXES)?;
        // for each possible target, we need to check if we have all of the required files for it
        for possible_target in possible_coverage_targets {
            let required_coverage_files = required_target_files(
                table_dir,
                &possible_target,
                COVERAGE_REQUIRED_TABLE_SUFFIXES,
            );
            if validate_target_files(&possible_target, required_coverage_files, "coverage") {
                plot_targets.coverage.insert(possible_target);
            }
        }
    }

    if plot_toggles.clustermap {
        // we only need to check the matrix directory for targets, since empty
        // variants files will be created for each target even if there's no matrix
        for matrix_type in matrix_types.enabled_matrix_types() {
            let possible_clustermap_targets =
                discover_candidate_targets(matrix_dir, &[matrix_type.file_suffix()])?;

            for possible_target in possible_clustermap_targets {
                // build up a list of theoretical paths, both from the matrix
                // directory and table directory, that all need to exist to create
                // the given target's clustermap
                let mut required = required_target_files(
                    table_dir,
                    &possible_target,
                    CLUSTERMAP_REQUIRED_TABLE_SUFFIXES,
                );
                required.push(
                    matrix_dir.join(format!("{possible_target}{}", matrix_type.file_suffix())),
                );

                if validate_target_files(&possible_target, required, "clustermap") {
                    plot_targets.clustermap.insert(matrix_type, possible_target);
                }
            }
        }
    }
    Ok(plot_targets)
}

/// Takes a path to a directory and a list of suffixes and returns a BTreeSet
/// of possible targets that have files with these suffixes
fn discover_candidate_targets(dir: &Path, suffixes: &[&str]) -> Result<BTreeSet<String>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Error reading input directory '{}'", dir.display()))?;
    let mut targets = BTreeSet::new();

    for entry in entries {
        let entry =
            entry.with_context(|| format!("Error reading entry from '{}'", dir.display()))?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };

        for suffix in suffixes {
            if let Some(target) = file_name.strip_suffix(suffix) {
                if is_valid_target_name(target) {
                    targets.insert(target.to_owned());
                } else {
                    eprintln!(
                        "Warning: skipping target derived from {file_name:?}: invalid target name {target:?}"
                    );
                }
                break;
            }
        }
    }

    Ok(targets)
}

/// Creates a list of theoretical paths for required target files to make a
/// certain plot, given the file suffixes for that plot type
fn required_target_files(dir: &Path, target: &str, suffixes: &[&str]) -> Vec<PathBuf> {
    suffixes
        .iter()
        .map(|suffix| dir.join(format!("{target}{suffix}")))
        .collect()
}

/// Checks if the required files exist to create a coverage plot for the given
/// target, based on a Vec of theoretical paths
fn validate_target_files(target: &str, required_files: Vec<PathBuf>, plot_type: &str) -> bool {
    let mut missing_files = Vec::new();

    for path in required_files {
        if !path.is_file() {
            missing_files.push(path);
            continue;
        }
    }

    if missing_files.is_empty() {
        return true;
    }

    // The existence of clustermap matrices is dependent on the data, and it is
    // quite likely that for some segments there is a clustermap and some there
    // is not, even if clustermap is enabled for the entire run.
    if plot_type != "clustermap" {
        eprintln!(
            "Could not create {plot_type} plot for {target}; missing required files: {}",
            missing_files
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    false
}

fn is_valid_target_name(target: &str) -> bool {
    !target.is_empty()
        && target.len() <= 128
        && target
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
