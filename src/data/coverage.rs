use crate::data::*;
use anyhow::{Result, anyhow};
use std::path::Path;

/// TODO: Docs
#[derive(Debug, serde::Deserialize)]
struct CoverageLine {
    #[serde(rename = "Position", deserialize_with = "option_float")]
    pub position: Option<f64>,
    #[serde(rename = "Coverage Depth", deserialize_with = "option_float")]
    pub coverage: Option<f64>,
}

/// TODO: Docs
#[derive(Debug, Clone)]
pub struct Coverage {
    pub position: Vec<f64>,
    pub coverage: Vec<f64>,
}

impl Coverage {
    /// TODO: Docs
    pub fn import_from_file(filename: &Path) -> Result<Self> {
        let mut coverage_data = Coverage {
            position: Vec::new(),
            coverage: Vec::new(),
        };

        let mut coverage_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in coverage_reader.deserialize() {
            let line: CoverageLine = line?;

            match (line.position, line.coverage) {
                (Some(pos), Some(cov)) => {
                    coverage_data.position.push(pos);
                    coverage_data.coverage.push(cov);
                }
                _ => continue,
            }
        }

        if coverage_data.coverage.is_empty() || coverage_data.position.is_empty() {
            return Err(anyhow!("File has no data."));
        }

        Ok(coverage_data)
    }
}
