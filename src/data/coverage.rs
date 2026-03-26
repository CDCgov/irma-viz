use crate::data::*;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
pub struct CoverageLine {
    #[serde(rename = "Reference_Name")]
    pub _reference_name: String,
    #[serde(rename = "Position")]
    pub _position: usize,
    #[serde(rename = "Coverage Depth")]
    pub coverage: usize,
    #[serde(rename = "Consensus", deserialize_with = "option_allele_byte")]
    pub _consenesus: Option<u8>,
    #[serde(rename = "Deletions")]
    pub _deletions: usize,
    #[serde(rename = "Ambiguous")]
    pub _ambiguous: usize,
    #[serde(rename = "Consensus_Count")]
    pub _consensus_count: usize,
    #[serde(rename = "Consensus_Average_Quality")]
    pub _consensus_aq: f64,
}

#[derive(Debug)]
pub struct CoverageData {
    pub coverage_vec: Vec<CoverageLine>,
}

impl CoverageData {
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut coverage_vec = Vec::new();

        let mut coverage_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in coverage_reader.deserialize() {
            let line: CoverageLine = line?;
            coverage_vec.push(line);
        }

        Ok(CoverageData { coverage_vec })
    }
}
