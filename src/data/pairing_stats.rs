//! Parsing for target-specific IRMA `*-pairingStats.txt` tables.

use std::{collections::HashMap, path::Path};

/// A row from a headerless IRMA `*-pairingStats.txt` table.
#[derive(serde::Deserialize)]
struct PairingStatsLine {
    _reference: String,
    key: String,
    value: f64,
}

/// Pairing statistics from `*-pairingStats.txt`, indexed by statistic name.
pub struct PairingStats {
    /// Values keyed by their IRMA statistic names.
    pub data: HashMap<String, f64>,
}

impl PairingStats {
    /// Reads a pairingStats TSV file and parses it, indexing each value by key.
    ///
    /// The table's reference column is ignored.
    ///
    /// ## Errors
    ///
    /// Returns an IO error if the csv reader is unable to be built, or a line
    /// of the csv reader is unable to be parsed
    pub fn import_from_file(filename: &Path) -> std::io::Result<Self> {
        let mut data = HashMap::new();

        let mut pairing_stats_reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in pairing_stats_reader.deserialize() {
            let line: PairingStatsLine = line?;

            data.insert(line.key, line.value);
        }

        Ok(PairingStats { data })
    }
}
