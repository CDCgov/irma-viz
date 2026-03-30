use std::{collections::HashMap, path::PathBuf};

#[derive(serde::Deserialize)]
/// TODO: Docs
struct PairingStatsLine {
    _reference: String,
    key: String,
    value: f64,
}

/// TODO: Docs
pub struct PairingStats {
    pub data: HashMap<String, f64>,
}

impl PairingStats {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
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
