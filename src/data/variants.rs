use crate::data::*;
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct AllVariants {
    pub positions: Vec<usize>,
    pub consensus_alleles: Vec<char>,
    pub minority_alleles: Vec<char>,
    pub minority_frequencies: MinorityFrequencies,
}

impl AllVariants {
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

#[derive(Debug, Clone)]
pub struct MinorityFrequencies {
    pub data: Vec<f64>,
    pub min: f64,
    pub max: f64,
}
