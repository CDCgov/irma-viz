//! Discovery and validation of targets with the inputs required for each plot.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{ParsedConfig, matrices::MatrixType},
    diagnostics::{PlotError, Severity, warn},
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
    /// Inserts a a matrix type into the [`BTreeSet`]
    fn insert(&mut self, matrix_type: MatrixType, target: String) {
        self.targets_for_mut(matrix_type).insert(target);
    }

    /// Gets the set of targets for a given matrix type
    pub fn targets_for(&self, matrix_type: MatrixType) -> &BTreeSet<String> {
        match matrix_type {
            MatrixType::Expenrd => &self.expenrd,
            MatrixType::Jaccard => &self.jaccard,
            MatrixType::Mutuald => &self.mutuald,
            MatrixType::Njointp => &self.njointp,
        }
    }

    /// Gets the set of targets for a given matrix type as mutable references
    /// for editing
    fn targets_for_mut(&mut self, matrix_type: MatrixType) -> &mut BTreeSet<String> {
        match matrix_type {
            MatrixType::Expenrd => &mut self.expenrd,
            MatrixType::Jaccard => &mut self.jaccard,
            MatrixType::Mutuald => &mut self.mutuald,
            MatrixType::Njointp => &mut self.njointp,
        }
    }

    /// Returns the union of all targets that have at least one discovered
    /// clustermap matrix file across the enabled matrix types.
    pub fn variant_targets(&self) -> BTreeSet<String> {
        self.expenrd
            .union(&self.jaccard)
            .chain(self.mutuald.union(&self.njointp))
            .cloned()
            .collect()
    }
}

/// Takes a directory and finds all possible heuristics targets from the
/// filenames. If a possible target is discovered, but is found to be invalid,
/// it will be skipped and a warning will be created by
/// [`validate_target_files`]
///
/// ## Errors
///
/// Will pass up IO errors that arise in [`discover_candidate_targets`]
pub fn discover_heuristics_targets(cfg: &ParsedConfig) -> Result<BTreeSet<String>, PlotError> {
    let mut heuristics_targets = BTreeSet::new();
    let possible_heuristics_targets =
        discover_candidate_targets(&cfg.io_args.table_path, HEURISTICS_REQUIRED_SUFFIXES)?;

    for possible_target in possible_heuristics_targets {
        let required_heuristics_files = required_target_files(
            &cfg.io_args.table_path,
            &possible_target,
            HEURISTICS_REQUIRED_SUFFIXES,
        );
        if validate_target_files(&possible_target, required_heuristics_files, "heuristics") {
            heuristics_targets.insert(possible_target);
        }
    }
    Ok(heuristics_targets)
}

/// Takes a directory and finds all possible coverage targets from the
/// filenames. If a possible target is discovered, but is found to be invalid,
/// it will be skipped and a warning will be created by
/// [`validate_target_files`]
///
/// A valid coverage target includes the following files
/// - {target}-variants.txt
/// - {target}-coverage.txt
/// - {target}-pairingStats.txt
///
/// ## Errors
///
/// Will pass up IO errors that arise in [`discover_candidate_targets`]
pub fn discover_coverage_targets(cfg: &ParsedConfig) -> Result<BTreeSet<String>, PlotError> {
    let mut coverage_targets = BTreeSet::new();
    // all of the potential targets we see
    let possible_coverage_targets =
        discover_candidate_targets(&cfg.io_args.table_path, COVERAGE_REQUIRED_TABLE_SUFFIXES)?;

    // for each possible target, we need to check if we have all of the required files for it
    for possible_target in possible_coverage_targets {
        let required_coverage_files = required_target_files(
            &cfg.io_args.table_path,
            &possible_target,
            COVERAGE_REQUIRED_TABLE_SUFFIXES,
        );
        if validate_target_files(&possible_target, required_coverage_files, "coverage") {
            coverage_targets.insert(possible_target);
        }
    }
    Ok(coverage_targets)
}

/// Takes a directory and finds all possible clustermap targets for each matrix
/// type from the filenames. If a possible target is discovered, but is found to
/// be invalid, it will be skipped and a warning will be created by
/// [`validate_target_files`]
///
/// A valid clustermap target includes the following files
/// - {target}-variants.txt
/// - {target}-{MatrixType}.txt
///
/// ## Errors
///
/// Will pass up IO errors that arise in [`discover_candidate_targets`]
pub fn discover_clustermap_targets(cfg: &ParsedConfig) -> Result<ClusterTargets, PlotError> {
    let mut clustermap_targets = ClusterTargets::default();

    for matrix_type in cfg
        .plot_specific
        .cluster_config
        .matrix_types
        .enabled_matrix_types()
    {
        let possible_clustermap_targets =
            discover_candidate_targets(&cfg.io_args.matrix_path, &[matrix_type.file_suffix()])?;

        for possible_target in possible_clustermap_targets {
            // build up a list of theoretical paths, both from the matrix
            // directory and table directory, that all need to exist to create
            // the given target's clustermap
            let mut required = required_target_files(
                &cfg.io_args.table_path,
                &possible_target,
                CLUSTERMAP_REQUIRED_TABLE_SUFFIXES,
            );
            required.push(
                cfg.io_args
                    .matrix_path
                    .join(format!("{possible_target}{}", matrix_type.file_suffix())),
            );

            if validate_target_files(
                &possible_target,
                required,
                &format!("{}_clustermap", matrix_type.display_name()),
            ) {
                clustermap_targets.insert(matrix_type, possible_target);
            }
        }
    }
    Ok(clustermap_targets)
}

/// Takes a path to a directory and a list of suffixes and returns a BTreeSet of
/// possible targets that have files with these suffixes. Will provide a warning
/// if a discovered potential target file is deemed invalid through
/// [`is_valid_target_name`]
///
/// ## Errors
///
/// Can pass up an IO Error from reading the directory if
/// - The provided path doesn't exist.
/// - The process lacks permissions to view the contents.
/// - The path points at a non-directory file.
///
/// Can pass up an IO Error if an error occurred while fetching files from the
/// OS
fn discover_candidate_targets(
    dir: &Path,
    suffixes: &[&str],
) -> Result<BTreeSet<String>, PlotError> {
    let entries = fs::read_dir(dir)
        .map_err(|err| PlotError::IOError(format!("reading directory '{}'", dir.display()), err))?;
    let mut targets = BTreeSet::new();

    for entry in entries {
        let entry = entry.map_err(|err| {
            PlotError::IOError(format!("reading entry from '{}'", dir.display()), err)
        })?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };

        for suffix in suffixes {
            if let Some(target) = file_name.strip_suffix(suffix) {
                if is_valid_target_name(target) {
                    targets.insert(target.to_owned());
                } else {
                    warn(
                        Severity::Warning,
                        format!(
                            "skipping target derived from {file_name:?}: invalid target name {target:?}"
                        ),
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

/// Checks whether all required files exist for a target-specific plot.
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

    warn(
        Severity::Warning,
        format!(
            "could not create {plot_type} plot for {target}; missing required files: {}",
            missing_files
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );

    false
}

/// Checks that a filename is not empty, is less than 128 characters, and only
/// contains ascii alphanumeric characters and `_`, `-`, and `.` characters.
pub fn is_valid_target_name(target: &str) -> bool {
    !target.is_empty()
        && target.len() <= 128
        && target
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
