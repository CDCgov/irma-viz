use std::{collections::HashMap, path::PathBuf};

struct PairingStatsLine {
    key: String,
    value: f64,
}

pub struct PairingStats {
    pub data: HashMap<String, f64>,
}

impl PairingStats {
    fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut data = HashMap::new();
        Ok(PairingStats { data })
    }
}
