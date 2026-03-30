use crate::data::*;
use std::{collections::HashMap, path::PathBuf};

#[derive(serde::Deserialize)]
pub struct VariantsLine {
    #[serde(rename = "Position")]
    position: usize,
    #[serde(rename = "Consensus_Allele", deserialize_with = "option_allele_byte")]
    conesensus_allele: Option<u8>,
    #[serde(rename = "Minority_Allele", deserialize_with = "option_allele_byte")]
    minority_allele: Option<u8>,
}

pub struct Variants {
    pub data: HashMap<usize, (Option<u8>, Option<u8>)>,
}

impl Variants {
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut data = HashMap::new();

        let mut variants_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in variants_reader.deserialize() {
            let line: VariantsLine = line?;

            data.insert(
                line.position,
                (line.conesensus_allele, line.minority_allele),
            );
        }

        Ok(Variants { data })
    }
}
