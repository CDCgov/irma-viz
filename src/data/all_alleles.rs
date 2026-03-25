use crate::data::*;
use std::path::PathBuf;

/// TODO: Docs
#[derive(serde::Deserialize)]
struct AllAllelesLine {
    #[serde(rename = "Reference_Name")]
    _reference_name: String,
    #[serde(rename = "Position")]
    _position: usize,
    #[serde(rename = "Allele", deserialize_with = "option_allele_byte")]
    _allele: Option<u8>,
    #[serde(rename = "Count")]
    _count: usize,
    #[serde(rename = "Total")]
    _total: usize,
    #[serde(rename = "Frequency")]
    frequency: f64,
    #[serde(rename = "Average_Quality", deserialize_with = "option_float")]
    average_quality: Option<f64>,
    #[serde(rename = "ConfidenceNotMacErr", deserialize_with = "option_float")]
    _confidence_not_mac_err: Option<f64>,
    #[serde(rename = "PairedUB")]
    _paired_ub: f64,
    #[serde(rename = "QualityUB")]
    _quality_ub: f64,
    #[serde(rename = "Allele_Type", deserialize_with = "is_consensus")]
    _is_consensus: bool,
}
/// TODO: Docs
pub struct AllAllelesData {
    pub _reference_names: Vec<String>,
    pub _positions: Vec<usize>,
    pub _alleles: Vec<Option<u8>>,
    pub _counts: Vec<usize>,
    pub _totals: Vec<usize>,
    pub frequencies: Vec<f64>,
    pub average_qualities: Vec<Option<f64>>,
    pub _confidence_not_mac_errs: Vec<Option<f64>>,
    pub _paried_ubs: Vec<f64>,
    pub _quality_ubs: Vec<f64>,
    pub _is_consensus: Vec<bool>,
}

impl AllAllelesData {
    /// TODO: Docs
    pub fn import_from_file(filename: &PathBuf) -> std::io::Result<Self> {
        let mut all_alleles_data = AllAllelesData {
            _reference_names: Vec::new(),
            _positions: Vec::new(),
            _alleles: Vec::new(),
            _counts: Vec::new(),
            _totals: Vec::new(),
            frequencies: Vec::new(),
            average_qualities: Vec::new(),
            _confidence_not_mac_errs: Vec::new(),
            _paried_ubs: Vec::new(),
            _quality_ubs: Vec::new(),
            _is_consensus: Vec::new(),
        };

        let mut all_alleles_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(filename)?;

        for line in all_alleles_reader.deserialize() {
            let line: AllAllelesLine = line?;

            all_alleles_data._reference_names.push(line._reference_name);
            all_alleles_data._positions.push(line._position);
            all_alleles_data._alleles.push(line._allele);
            all_alleles_data._counts.push(line._count);
            all_alleles_data._totals.push(line._total);
            all_alleles_data.frequencies.push(line.frequency);
            all_alleles_data
                .average_qualities
                .push(line.average_quality);
            all_alleles_data
                ._confidence_not_mac_errs
                .push(line._confidence_not_mac_err);
            all_alleles_data._paried_ubs.push(line._paired_ub);
            all_alleles_data._quality_ubs.push(line._quality_ub);
            all_alleles_data._is_consensus.push(line._is_consensus);
        }

        Ok(all_alleles_data)
    }
}
