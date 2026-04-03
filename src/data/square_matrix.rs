use anyhow::{Context, anyhow};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug, Clone)]
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
        let mut row_num = 0;
        for (line_num, line) in sqm_reader.enumerate() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let (label, row) = Self::parse_line(&line)
                .with_context(|| format!("Failed to parse line number {ln}", ln = line_num + 1))?;

            labels.push(label);
            matrix.push(row);

            let prev_row = line_num.saturating_sub(1);
            if matrix[prev_row].len() != matrix[row_num].len() {
                return Err(anyhow!("Matrix is not square."));
            }
            row_num += 1;
        }

        if matrix.is_empty() {
            return Err(anyhow!("Matrix is empty."));
        } else if matrix.len() != matrix[0].len() {
            // No indexing panic, checks for empty matrix in previous arm.
            // All rows should be equal length based on earlier check.
            return Err(anyhow!("Matrix is not square."));
        }

        Ok(SquareMatrix { labels, matrix })
    }

    /// TODO: Docs
    fn parse_line(line: &str) -> anyhow::Result<(String, Vec<f64>)> {
        let mut split_line = line.split('\t');

        // No panic: line is checked for empty before passed to this func
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
