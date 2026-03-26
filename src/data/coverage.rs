use std::path::PathBuf;

// TODO: find out what is needed for `coverageDiagram.r`
// Can also change type to float if needed for kuva to avoid later casting
#[derive(Debug, serde::Deserialize)]
/// TODO: Docs
pub struct CoverageLine {
    #[serde(rename = "Position")]
    pub position: usize,
    #[serde(rename = "Coverage Depth")]
    pub coverage: usize,
}

#[derive(Debug)]
/// TODO: Docs
pub struct CoverageData {
    pub positions: Vec<usize>,
    pub coverages: Vec<usize>,
}

impl CoverageData {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut coverage_data = CoverageData {
            positions: Vec::new(),
            coverages: Vec::new(),
        };

        let mut coverage_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in coverage_reader.deserialize() {
            let line: CoverageLine = line?;
            coverage_data.positions.push(line.position);
            coverage_data.coverages.push(line.coverage)
        }

        Ok(coverage_data)
    }
}
