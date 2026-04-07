use crate::data::*;
use anyhow::anyhow;
use std::path::PathBuf;

// TODO: find out what is needed for `coverageDiagram.r`
// Can also change type to float if needed for kuva to avoid later casting
#[derive(Debug, serde::Deserialize)]
/// TODO: Docs
struct CoverageLine {
    #[serde(rename = "Position", deserialize_with = "option_float")]
    pub position: Option<f64>,
    #[serde(rename = "Coverage Depth", deserialize_with = "option_float")]
    pub coverage: Option<f64>,
    #[serde(rename = "Consensus", deserialize_with = "option_allele_byte")]
    pub consensus: Option<u8>,
}

#[derive(Debug, Clone)]
/// TODO: Docs
pub struct Coverage {
    pub coverage_by_position: Vec<(f64, f64)>,
    pub consensuses: Vec<Option<u8>>,
}

impl Coverage {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> anyhow::Result<Self> {
        let mut coverage_data = Coverage {
            coverage_by_position: Vec::new(),
            consensuses: Vec::new(),
        };

        let mut coverage_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in coverage_reader.deserialize() {
            let line: CoverageLine = line?;

            match (line.position, line.coverage) {
                (Some(pos), Some(cov)) => {
                    coverage_data.coverage_by_position.push((pos, cov));
                    coverage_data.consensuses.push(line.consensus);
                }
                _ => continue,
            }
        }

        if coverage_data.coverage_by_position.is_empty() || coverage_data.consensuses.is_empty() {
            return Err(anyhow!("File has no data."));
        }

        Ok(coverage_data)
    }
}
