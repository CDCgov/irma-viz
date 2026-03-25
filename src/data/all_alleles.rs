use serde::{Deserialize, de::Error};
use std::path::PathBuf;

/// TODO: Docs
#[derive(serde::Deserialize)]
struct AllAllelesLine {
    #[serde(rename = "Reference_Name")]
    reference_name: String,
    #[serde(rename = "Position")]
    position: usize,
    #[serde(rename = "Allele", deserialize_with = "option_allele_byte")]
    allele: Option<u8>,
    #[serde(rename = "Count")]
    count: usize,
    #[serde(rename = "Total")]
    total: usize,
    #[serde(rename = "Frequency")]
    frequency: f64,
    #[serde(rename = "Average_Quality", deserialize_with = "option_float")]
    average_quality: Option<f64>,
    #[serde(rename = "ConfidenceNotMacErr", deserialize_with = "option_float")]
    confidence_not_mac_err: Option<f64>,
    #[serde(rename = "PairedUB")]
    paired_ub: f64,
    #[serde(rename = "QualityUB")]
    quality_ub: f64,
    #[serde(rename = "Allele_Type", deserialize_with = "is_consensus")]
    is_consensus: bool,
}

/// TODO: Docs
fn option_allele_byte<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "-" => Ok(None),
        "A" | "C" | "G" | "T" => Ok(Some(s.as_bytes()[0])),
        _ => Err(D::Error::custom(
            "Failed to parse Allele field. Allele is not \"A\", \"C\", \"G\", \"T\", or \"-\".",
        )),
    }
}

/// TODO: Docs
fn option_float<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "NA" => Ok(None),
        _ => s.parse::<f64>().map(Some).map_err(D::Error::custom),
    }
}

/// TODO: Docs
fn is_consensus<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    match s {
        "Consensus" => Ok(true),
        "Minority" => Ok(false),
        _ => Err(D::Error::custom(
            "Failed to parse Allele Type. Allele Type is not \"Consensus\" or \"Minority\".",
        )),
    }
}

/// TODO: Docs
pub struct AllAllelesData {
    pub reference_names: Vec<String>,
    pub positions: Vec<usize>,
    pub alleles: Vec<Option<u8>>,
    pub counts: Vec<usize>,
    pub totals: Vec<usize>,
    pub frequencies: Vec<f64>,
    pub average_qualities: Vec<Option<f64>>,
    pub confidence_not_mac_errs: Vec<Option<f64>>,
    pub paried_ubs: Vec<f64>,
    pub quality_ubs: Vec<f64>,
    pub is_consensus: Vec<bool>,
}

impl AllAllelesData {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut all_alleles_data = AllAllelesData {
            reference_names: Vec::new(),
            positions: Vec::new(),
            alleles: Vec::new(),
            counts: Vec::new(),
            totals: Vec::new(),
            frequencies: Vec::new(),
            average_qualities: Vec::new(),
            confidence_not_mac_errs: Vec::new(),
            paried_ubs: Vec::new(),
            quality_ubs: Vec::new(),
            is_consensus: Vec::new(),
        };

        let mut all_alleles_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in all_alleles_reader.deserialize() {
            let line: AllAllelesLine = line?;

            all_alleles_data.reference_names.push(line.reference_name);
            all_alleles_data.positions.push(line.position);
            all_alleles_data.alleles.push(line.allele);
            all_alleles_data.counts.push(line.count);
            all_alleles_data.totals.push(line.total);
            all_alleles_data.frequencies.push(line.frequency);
            all_alleles_data
                .average_qualities
                .push(line.average_quality);
            all_alleles_data
                .confidence_not_mac_errs
                .push(line.confidence_not_mac_err);
            all_alleles_data.paried_ubs.push(line.paired_ub);
            all_alleles_data.quality_ubs.push(line.quality_ub);
            all_alleles_data.is_consensus.push(line.is_consensus);
        }

        Ok(all_alleles_data)
    }
}
