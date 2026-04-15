use crate::data::*;
use std::path::Path;

#[derive(serde::Deserialize)]
struct VariantsLine {
    #[serde(rename = "Position")]
    position: f64,
    #[serde(rename = "Consensus_Allele", deserialize_with = "option_allele_byte")]
    consensus_allele: Option<u8>,
    #[serde(rename = "Minority_Allele", deserialize_with = "option_allele_byte")]
    minority_allele: Option<u8>,
    #[serde(rename = "Minority_Frequency")]
    minority_frequency: f64,
}
pub struct Variants {
    pub data: Vec<(f64, Option<u8>, Option<u8>, f64)>,
}

impl Variants {
    pub fn import_from_file(filename: &Path) -> std::io::Result<Self> {
        let mut data = Vec::new();

        let mut variants_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in variants_reader.deserialize() {
            let line: VariantsLine = line?;

            data.push((
                line.position,
                line.consensus_allele,
                line.minority_allele,
                line.minority_frequency,
            ));
        }

        Ok(Variants { data })
    }
}
