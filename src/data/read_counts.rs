//! Parsing for run-level IRMA `READ_COUNTS.txt` tables.

use crate::data::option_float;
use std::{collections::HashMap, path::Path};

/// A deserialized row from the run-level `READ_COUNTS.txt` table.
#[derive(Debug, serde::Deserialize)]
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

/// Read-count metrics from `READ_COUNTS.txt`, indexed by IRMA record name.
#[derive(Debug)]
pub struct ReadCounts {
    pub map: HashMap<String, Data>,
}

/// Counts associated with one `READ_COUNTS.txt` record.
#[derive(Debug)]
pub struct Data {
    pub read: f64,
    pub pattern: Option<f64>,
    pub pairs_and_windows: Option<f64>,
}

impl ReadCounts {
    #[allow(unused)]
    /// Reads a headered `READ_COUNTS.txt` TSV and indexes rows by `Record`.
    ///
    /// `NA` values in `Patterns` and `PairsAndWidows` are represented as
    /// [`None`].
    ///
    /// ## Errors
    ///
    /// Will return an IO error if the CSV reader cannot be built, or a line
    /// cannot be deserialized
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

    /// Returns the read count for `key`, or `0.0` when it is absent.
    pub fn read(&self, key: &str) -> f64 {
        match self.map.get(key) {
            Some(data) => data.read,
            None => 0.0,
        }
    }

    /// Returns the pattern count for `key`, mapping absent or `NA` values to
    /// `0.0`.
    pub fn pattern(&self, key: &str) -> f64 {
        match self.map.get(key) {
            Some(data) => data.pattern.unwrap_or(0.0),
            None => 0.0,
        }
    }

    /// Returns the PairsAndWidows count for `key`, mapping absent or `NA`
    /// values to `0.0`.
    pub fn pairs_and_widows(&self, key: &str) -> f64 {
        match self.map.get(key) {
            Some(data) => data.pairs_and_windows.unwrap_or(0.0),
            None => 0.0,
        }
    }
}

/// Directed `(source, target, read_count)` links for a Sankey figure, based on
/// the format they are expected by Kuva
pub struct SankeyVec {
    pub edges: Vec<(String, String, f64)>,
}

impl SankeyVec {
    /// Reads in a READ_COUNTS file and converts it into a parsed [`SankeyVec`]
    /// format
    ///
    /// Stages `0` and `1` are ignored. Unrecognized records outside stages
    /// `4` and `5` cause an error.
    ///
    /// ## Errors
    ///
    /// Can return an IO error if the csv reader is unable to be built from the
    /// provided path, or if an error occurs while reading a line of csv data
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
                "3-unrecognizable" => (String::from("Pass QC"), String::from("Unrecognized"), read),
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
