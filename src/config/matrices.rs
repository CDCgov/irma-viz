//! IRMA matrix types available for clustermap figures.

use serde::Deserialize;

/// Toggles for individual IRMA matrix types for clustermaps.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct MatrixTypes {
    pub expenrd: bool,
    pub jaccard: bool,
    pub mutuald: bool,
    pub njointp: bool,
}

/// IRMA matrix types
#[derive(Debug, Clone, Copy)]
pub enum MatrixType {
    Expenrd,
    Jaccard,
    Mutuald,
    Njointp,
}

impl MatrixTypes {
    /// Converts the struct of bools created from the config.toml into a Vec of
    /// enabled matrix types for iteration
    pub fn enabled_matrix_types(&self) -> Vec<MatrixType> {
        let mut enabled = Vec::new();
        if self.expenrd {
            enabled.push(MatrixType::Expenrd);
        }
        if self.jaccard {
            enabled.push(MatrixType::Jaccard);
        }
        if self.mutuald {
            enabled.push(MatrixType::Mutuald);
        }
        if self.njointp {
            enabled.push(MatrixType::Njointp);
        }
        enabled
    }
}

impl MatrixType {
    /// Returns the uppercase Matrix name for filenames and output messages
    pub fn display_name(self) -> &'static str {
        match self {
            MatrixType::Expenrd => "EXPENRD",
            MatrixType::Jaccard => "JACCARD",
            MatrixType::Mutuald => "MUTUALD",
            MatrixType::Njointp => "NJOINTP",
        }
    }

    /// Returns the IRMA filename suffix for the given matrix
    pub fn file_suffix(self) -> &'static str {
        match self {
            MatrixType::Expenrd => "-EXPENRD.sqm",
            MatrixType::Jaccard => "-JACCARD.sqm",
            MatrixType::Mutuald => "-MUTUALD.sqm",
            MatrixType::Njointp => "-NJOINTP.sqm",
        }
    }
}
