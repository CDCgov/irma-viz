//! Parsing for target-specific IRMA `*-variants.txt` tables.

use crate::data::*;
use std::path::PathBuf;

/// A deserialized minority-variant row from an IRMA `*-variants.txt` table.
#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct Variant {
    #[serde(rename = "Position")]
    pub position: usize,
    #[serde(rename = "Consensus_Allele", deserialize_with = "allele_char")]
    pub consensus_allele: char,
    #[serde(rename = "Minority_Allele", deserialize_with = "allele_char")]
    pub minority_allele: char,
    #[serde(rename = "Minority_Frequency")]
    pub minority_frequency: f64,
}

/// Column-oriented minority-variant data for one IRMA target.
///
/// All parallel vectors preserve source-file order. Frequency values and their
/// observed bounds are stored in [`MinorityFrequencies`].
#[derive(Debug, Clone)]
pub struct AllVariants {
    /// Variant positions in source-file order.
    pub positions: Vec<usize>,
    /// Consensus alleles corresponding to [`AllVariants::positions`].
    pub consensus_alleles: Vec<char>,
    /// Minority alleles corresponding to [`AllVariants::positions`].
    pub minority_alleles: Vec<char>,
    /// Minority frequencies corresponding to [`AllVariants::positions`].
    pub minority_frequencies: MinorityFrequencies,
}

impl AllVariants {
    /// Reads a target's `*-variants.txt` TSV into column-oriented variant data
    /// for a coverage plot.
    ///
    /// ## Errors
    ///
    /// Returns an IO error if the csv reader is unable to be built, or a line
    /// of the csv reader is unable to be parsed
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut variants = AllVariants {
            positions: Vec::new(),
            consensus_alleles: Vec::new(),
            minority_alleles: Vec::new(),
            minority_frequencies: MinorityFrequencies {
                data: Vec::new(),
                min: f64::MAX,
                max: f64::MIN,
            },
        };

        let mut variants_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in variants_reader.deserialize() {
            let variant: Variant = line?;

            variants.positions.push(variant.position);
            variants.consensus_alleles.push(variant.consensus_allele);
            variants.minority_alleles.push(variant.minority_allele);
            variants
                .minority_frequencies
                .data
                .push(variant.minority_frequency);
            if variant.minority_frequency > variants.minority_frequencies.max {
                variants.minority_frequencies.max = variant.minority_frequency;
            }
            if variant.minority_frequency < variants.minority_frequencies.min {
                variants.minority_frequencies.min = variant.minority_frequency;
            }
        }

        Ok(variants)
    }
}

/// Minority-allele frequencies and extrema accumulated during parsing.
#[derive(Debug, Clone)]
pub struct MinorityFrequencies {
    /// Frequency values in source-file order.
    pub data: Vec<f64>,
    pub min: f64,
    pub max: f64,
}
