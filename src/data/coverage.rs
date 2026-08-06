use crate::{data::*, diagnostics::PlotError};
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
    /// Reads a coverage TSV file and parses it for a coverage plot.
    ///
    /// ## Errors
    ///
    /// Returns an IO error if the csv reader is unable to be built, or a line
    /// of the csv reader is unable to be parsed, or if the coverage data is
    /// empty
    pub fn import_from_file(filename: &Path) -> Result<Self, PlotError> {
        let mut coverage_data = Coverage {
            position: Vec::new(),
            coverage: Vec::new(),
        };

        let mut coverage_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)
            .map_err(|err| {
                PlotError::IOError(format!("opening '{}'", filename.display()), err.into())
            })?;

        for line in coverage_reader.deserialize() {
            let line: CoverageLine = line.map_err(|err| PlotError::InvalidData(err.to_string()))?;

            match (line.position, line.coverage) {
                (Some(pos), Some(cov)) => {
                    coverage_data.position.push(pos);
                    coverage_data.coverage.push(cov);
                }
                _ => continue,
            }
        }

        if coverage_data.coverage.is_empty() || coverage_data.position.is_empty() {
            return Err(PlotError::MissingData(format!(
                "{} is empty",
                filename.display()
            )));
        }

        Ok(coverage_data)
    }
}
