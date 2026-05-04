use crate::data::*;
use std::path::Path;

/// TODO: Docs
#[derive(serde::Deserialize)]
struct AllAllelesLine {
    #[serde(rename = "Total")]
    total: f64,
    #[serde(rename = "Frequency")]
    frequency: f64,
    #[serde(rename = "Average_Quality", deserialize_with = "option_float")]
    average_quality: Option<f64>,
    #[serde(rename = "ConfidenceNotMacErr", deserialize_with = "option_float")]
    confidence_not_mac_err: Option<f64>,
}
/// TODO: Docs
pub struct AllAlleles {
    pub totals: Vec<f64>,
    pub frequencies: Vec<f64>,
    pub average_qualities: AverageQualities,
    pub confidence_not_mac_errs: Vec<Option<f64>>,
}

impl AllAlleles {
    /// TODO: Docs
    pub fn import_from_file(filename: &Path) -> std::io::Result<Self> {
        let mut all_alleles_data = AllAlleles {
            totals: Vec::new(),
            frequencies: Vec::new(),
            average_qualities: AverageQualities {
                data: Vec::new(),
                min: f64::MAX,
                max: f64::MIN,
            },
            confidence_not_mac_errs: Vec::new(),
        };

        let mut all_alleles_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in all_alleles_reader.deserialize() {
            let line: AllAllelesLine = line?;

            all_alleles_data.totals.push(line.total);

            all_alleles_data.frequencies.push(line.frequency);

            if let Some(aq) = line.average_quality {
                all_alleles_data.average_qualities.data.push(aq);
                if aq > all_alleles_data.average_qualities.max {
                    all_alleles_data.average_qualities.max = aq;
                }
                if aq < all_alleles_data.average_qualities.min {
                    all_alleles_data.average_qualities.min = aq;
                }
            }
            all_alleles_data
                .confidence_not_mac_errs
                .push(line.confidence_not_mac_err);
        }

        Ok(all_alleles_data)
    }
}

pub struct AverageQualities {
    pub data: Vec<f64>,
    pub min: f64,
    pub max: f64,
}
