use crate::data::option_float;
use std::{collections::HashMap, path::Path};

#[derive(Debug, serde::Deserialize)]
/// TODO: Docs
struct ReadCountsLine {
    #[serde(rename = "Record")]
    pub record: String,
    #[serde(rename = "Reads")]
    pub read: f64,
    #[serde(rename = "Patterns", deserialize_with = "option_float")]
    pub pattern: Option<f64>,
    #[serde(rename = "PairsAndWidows", deserialize_with = "option_float")]
    pub p_a_w: Option<f64>,
}

#[derive(Debug)]
/// TODO: Docs
pub struct ReadCounts {
    pub map: HashMap<String, Data>,
}

#[derive(Debug)]
pub struct Data {
    pub read: f64,
    pub pattern: Option<f64>,
    pub pairs_and_windows: Option<f64>,
}

impl ReadCounts {
    #[allow(unused)]
    /// TODO: Docs
    pub fn import_from_file(filename: &Path) -> std::io::Result<Self> {
        let mut map = HashMap::new();

        let mut read_counts_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in read_counts_reader.deserialize() {
            let line: ReadCountsLine = line?;
            map.insert(
                line.record,
                Data {
                    read: line.read,
                    pattern: line.pattern,
                    pairs_and_windows: line.p_a_w,
                },
            );
        }

        Ok(ReadCounts { map })
    }

    pub fn read(&self, key: &str) -> f64 {
        match self.map.get(key) {
            Some(data) => data.read,
            None => 0.0,
        }
    }

    pub fn pattern(&self, key: &str) -> f64 {
        match self.map.get(key) {
            Some(data) => data.pattern.unwrap_or(0.0),
            None => 0.0,
        }
    }

    pub fn pairs_and_widows(&self, key: &str) -> f64 {
        match self.map.get(key) {
            Some(data) => data.pairs_and_windows.unwrap_or(0.0),
            None => 0.0,
        }
    }
}

pub struct SankeyVec {
    pub edges: Vec<(String, String, f64)>,
}

impl SankeyVec {
    pub fn import_from_file(filename: &Path) -> std::io::Result<Self> {
        let mut edges = Vec::new();

        let mut read_counts_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in read_counts_reader.deserialize() {
            let line: ReadCountsLine = line?;

            if line.record.starts_with("0-") || line.record.starts_with("1-") {
                continue;
            }

            let read = line.read;
            edges.push(match line.record.as_str() {
                "2-passQC" => (String::from("Initial Reads"), String::from("Pass QC"), read),
                "2-failQC" => (String::from("Initial Reads"), String::from("Fail QC"), read),
                "3-match" => (String::from("Pass QC"), String::from("Primary Match"), read),
                "3-nomatch" => (String::from("Pass QC"), String::from("No Match"), read),
                "3-altmatch" => (String::from("Pass QC"), String::from("Alt Match"), read),
                "3-chimeric" => (String::from("Pass QC"), String::from("Chimeric"), read),
                _ => {
                    // TODO: check matches vs targets list
                    if let Some(record) = line.record.as_str().strip_prefix("4-") {
                        (String::from("Primary Match"), String::from(record), read)
                    } else if let Some(record) = line.record.as_str().strip_prefix("5-") {
                        (String::from("Alt Match"), String::from(record), read)
                    } else {
                        return Err(std::io::Error::other(format!(
                            "Unrecognized value in Record field: \"{record}\"",
                            record = line.record
                        )));
                    }
                }
            })
        }

        Ok(SankeyVec { edges })
    }
}
