use crate::data::*;
use core::option::Option;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, serde::Deserialize)]
/// TODO: Docs
pub struct ReadCountsLine {
    #[serde(rename = "Record")]
    pub record: String,
    #[serde(rename = "Reads", deserialize_with = "option_float")]
    pub read: Option<f64>,
    #[serde(rename = "Patterns", deserialize_with = "option_float")]
    pub _pattern: Option<f64>,
    #[serde(rename = "PairsAndWindows", deserialize_with = "option_float")]
    pub _pair_and_window: Option<f64>,
}

#[derive(Debug)]
/// TODO: Docs
pub struct ReadCountsData {
    pub record_data_map: HashMap<String, ReadCountsLine>,
}

impl ReadCountsData {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut record_data_map = HashMap::new();

        let mut read_counts_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in read_counts_reader.deserialize() {
            let line: ReadCountsLine = line?;

            record_data_map.insert(line.record.clone(), line);
        }

        Ok(ReadCountsData { record_data_map })
    }
}
