use anyhow::Context;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug)]
/// TODO: Docs
pub struct SquareMatrix {
    pub labels: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

impl SquareMatrix {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> anyhow::Result<Self> {
        let mut labels = Vec::new();
        let mut matrix = Vec::new();

        let sqm_reader = BufReader::new(File::open(filename)?).lines();
        for (line_num, line) in sqm_reader.enumerate() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let (label, row) = Self::parse_line(&line)
                .with_context(|| format!("Failed to parse line number {ln}", ln = line_num + 1))?;

            labels.push(label);
            matrix.push(row);
        }

        Ok(SquareMatrix { labels, matrix })
    }

    /// TODO: Docs
    fn parse_line(line: &str) -> anyhow::Result<(String, Vec<f64>)> {
        let mut split_line = line.split('\t');

        let label = split_line.next().expect("Line should not be empty.");

        let row = split_line
            .map(|x| {
                x.parse::<f64>()
                    .with_context(|| format!("Unable to parse \"{x}\" as float."))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((label.to_string(), row))
    }
}
