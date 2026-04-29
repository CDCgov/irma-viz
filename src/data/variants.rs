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
    pub data: Vec<Variant>,
}

impl AllVariants {
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut data = Vec::new();

        let mut variants_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in variants_reader.deserialize() {
            let Variant {
                position,
                consensus_allele,
                minority_allele,
                minority_frequency,
            } = line?;

            data.push(Variant {
                position,
                consensus_allele,
                minority_allele,
                minority_frequency,
            });
        }

        Ok(AllVariants { data })
    }
}
