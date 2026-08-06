use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::diagnostics::PlotError;

/// Parsed square-matrix input used to render clustermap and tree plots.
#[derive(Debug, Clone)]
pub struct SquareMatrix {
    pub labels: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

impl SquareMatrix {
    /// Reads a square-matrix file and parses it into labels and values.
    ///
    /// ## Errors
    ///
    /// Returns an error if the file cannot be opened or read, if any row cannot
    /// be parsed into floats, if row lengths are inconsistent, or
    /// if the resulting matrix is empty or non-square.
    pub fn import_from_file(filename: &Path) -> Result<Self, PlotError> {
        let mut labels = Vec::new();
        let mut matrix = Vec::new();

        let sqm_reader =
            BufReader::new(File::open(filename).map_err(|err| {
                PlotError::IOError(format!("opening '{}'", filename.display()), err)
            })?)
            .lines();

        let mut expected_len = None;
        for (line_num, line) in sqm_reader.enumerate() {
            let line = line.map_err(|err| {
                PlotError::IOError(format!("reading '{}'", filename.display()), err)
            })?;
            if line.is_empty() {
                continue;
            }

            let (label, row) = Self::parse_line(&line).map_err(|err| {
                PlotError::InvalidData(format!(
                    "Failed to parse line {} in '{}': {err}",
                    line_num + 1,
                    filename.display()
                ))
            })?;

            match expected_len {
                None => expected_len = Some(row.len()),
                Some(len) if row.len() != len => {
                    return Err(PlotError::InvalidData(format!(
                        "Matrix in {} is not square.",
                        filename.display()
                    )));
                }
                _ => (),
            }

            labels.push(label);
            matrix.push(row);
        }

        if matrix.is_empty() {
            return Err(PlotError::MissingData(format!(
                "Square matrix in {} is empty.",
                filename.display()
            )));
        } else if matrix.len() != matrix[0].len() {
            // No indexing panic, checks for empty matrix in previous arm.
            // All rows should be equal length based on earlier check.
            return Err(PlotError::InvalidData(format!(
                "Matrix in {} is not square.",
                filename.display()
            )));
        }

        Ok(SquareMatrix { labels, matrix })
    }

    /// Parses one tab-delimited square-matrix row into its label and numeric
    /// values.
    ///
    /// ## Errors
    ///
    /// Returns an error if any matrix cell after the row label cannot be parsed
    /// as an `f64`.
    fn parse_line(line: &str) -> Result<(String, Vec<f64>), PlotError> {
        let mut split_line = line.split('\t');

        // No panic: line is checked for empty before passed to this func
        let label = split_line.next().expect("Line should not be empty.");

        let row = split_line
            .map(|x| {
                x.parse::<f64>().map_err(|err| {
                    PlotError::InvalidData(format!("Unable to parse \"{x}\" as float: {}", err))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((label.to_string(), row))
    }
}
