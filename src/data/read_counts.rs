use core::option::Option;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, prelude::*}, path::PathBuf,
};
use zoe::data::err::ResultWithErrorContext;

// Header format for `READ_COUNTS.txt`
// TODO: Docs
const READ_COUNTS_HEADER: &str = "Record\tReads\tPatterns\tPairsAndWidows";
const RECORD_COL: usize = 0;
const READS_COL: usize = 1;
const PATTERNS_COL: usize = 2;
const PAIRSANDWINDOWS_COL: usize = 3;
const MAX_COLS: usize = 4;

#[derive(Debug)]
/// TODO: Docs
pub struct ReadCountLine {
    pub record: String,
    pub _read: Option<usize>,
    pub _pattern: Option<usize>,
    pub _pair_and_window: Option<usize>,
}
#[derive(Debug)]
/// TODO: Docs
pub struct ReadCountsData {
    pub record_data_map: HashMap<String, ReadCountLine>,
}

impl ReadCountsData {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut read_counts_lines =
            BufReader::new(File::open(filename).with_context("Cannot open file")?).lines();

        let Some(read_counts_header) = read_counts_lines.next().transpose()? else {
            return Err(std::io::Error::other("File is empty."));
        };
        if read_counts_header != READ_COUNTS_HEADER {
            return Err(std::io::Error::other("Invalid header format."));
        }

        let mut record_data_map = HashMap::new();
        for (line_num, line) in read_counts_lines.enumerate() {
            let line = line?;

            if line.is_empty() {
                continue;
            }

            let read_count_line = Self::parse_line(line)
                .with_context(format!("Failed to parse line number {}", line_num + 1))?;

            record_data_map.insert(read_count_line.record.clone(), read_count_line);
        }

        Ok(ReadCountsData { record_data_map })
    }

    /// TODO: Docs
    fn parse_line(line: String) -> std::io::Result<ReadCountLine> {
        let split_line = line.split('\t').collect::<Vec<_>>();
        if split_line.len() < MAX_COLS {
            return Err(std::io::Error::other(format!(
                "Too few fields. Found: {found}, expected: {MAX_COLS}",
                found = split_line.len()
            )));
        }

        let record = split_line[RECORD_COL].to_string();
        let read = Some(
            split_line[READS_COL]
                .parse::<usize>()
                .with_context("Failed to parse Read field.")?,
        );
        let pattern = match split_line[PATTERNS_COL] {
            "NA" => None,
            _ => Some(
                split_line[PATTERNS_COL]
                    .parse::<usize>()
                    .with_context("Failed to parse Pattern field.")?,
            ),
        };
        let pair_and_window = match split_line[PAIRSANDWINDOWS_COL] {
            "NA" => None,
            _ => Some(
                split_line[PAIRSANDWINDOWS_COL]
                    .parse::<usize>()
                    .with_context("Failed to parse PairsAndWidows field.")?,
            ),
        };

        Ok(ReadCountLine {
            record,
            _read: read,
            _pattern: pattern,
            _pair_and_window: pair_and_window,
        })
    }
}
