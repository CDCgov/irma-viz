use std::{collections::HashMap, path::Path};

#[derive(Debug, serde::Deserialize)]
/// TODO: Docs
struct ReadCountsLine {
    #[serde(rename = "Record")]
    pub record: String,
    #[serde(rename = "Reads")]
    pub read: f64,
}

#[derive(Debug)]
/// TODO: Docs
pub struct ReadCounts {
    pub record_data_map: HashMap<String, f64>,
}

impl ReadCounts {
    #[allow(unused)]
    /// TODO: Docs
    pub fn import_from_file(filename: &Path) -> std::io::Result<Self> {
        let mut record_data_map = HashMap::new();

        let mut read_counts_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in read_counts_reader.deserialize() {
            let line: ReadCountsLine = line?;
            record_data_map.insert(line.record, line.read);
        }

        Ok(ReadCounts { record_data_map })
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
