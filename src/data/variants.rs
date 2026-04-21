use crate::data::*;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct VariantsLine {
    #[serde(rename = "Position")]
    position: usize,
    #[serde(rename = "Consensus_Allele", deserialize_with = "option_allele_byte")]
    consensus_allele: Option<u8>,
    #[serde(rename = "Minority_Allele", deserialize_with = "option_allele_byte")]
    minority_allele: Option<u8>,
    #[serde(rename = "Minority_Frequency")]
    minority_frequency: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub position: usize,
    pub consensus_allele: char,
    pub minority_allele: char,
    pub minority_frequency: f64,
}
#[derive(Debug, Clone)]
pub struct Variants {
    pub data: Vec<Variant>,
}

impl Variants {
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut data = Vec::new();

        let mut variants_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in variants_reader.deserialize() {
            let VariantsLine {
                position,
                consensus_allele,
                minority_allele,
                minority_frequency,
            } = line?;

            let consensus_allele = consensus_allele.unwrap_or(b'N') as char;
            let minority_allele = minority_allele.unwrap_or(b'N') as char;

            data.push(Variant {
                position,
                consensus_allele,
                minority_allele,
                minority_frequency,
            });
        }

        Ok(Variants { data })
    }
}
