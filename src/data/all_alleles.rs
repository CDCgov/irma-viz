use crate::data::*;
use std::path::PathBuf;

/// TODO: Docs
#[derive(serde::Deserialize)]
struct AllAllelesLine {
    #[serde(rename = "Frequency")]
    frequency: f64,
    #[serde(rename = "Average_Quality", deserialize_with = "option_float")]
    average_quality: Option<f64>,
}
/// TODO: Docs
pub struct AllAllelesData {
    pub frequencies: Vec<f64>,
    pub average_qualities: Vec<Option<f64>>,
}

impl AllAllelesData {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut all_alleles_data = AllAllelesData {
            frequencies: Vec::new(),
            average_qualities: Vec::new(),
        };

        let mut all_alleles_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in all_alleles_reader.deserialize() {
            let line: AllAllelesLine = line?;

            all_alleles_data.frequencies.push(line.frequency);
            all_alleles_data
                .average_qualities
                .push(line.average_quality);
        }

        Ok(all_alleles_data)
    }
}
