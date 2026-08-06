use crate::{data::*, diagnostics::PlotError};
use std::path::Path;

const TOTAL_PROB: f64 = 0.2;

/// A deserialized row from an IRMA `*-allAlleles.txt` table.
/// Only the columns needed for the heuristics plot are represented here.
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
/// Parsed data from an IRMA `*-allAlleles.txt` table for one target.
/// The totals are trimmed to the low-coverage quantile used by the coverage
/// histogram, while frequencies, qualities, and confidence values are retained for their plots.
pub struct AllAlleles {
    pub totals: Totals,
    pub frequencies: Vec<f64>,
    pub average_qualities: AverageQualities,
    pub confidence_not_mac_errs: Vec<f64>,
}

impl AllAlleles {
    /// Reads an all-alleles TSV file and extracts the columns used by the
    /// heuristics figure. Missing quality/confidence values are skipped, zero
    /// confidence values are excluded, and totals are filtered to the
    /// configured quantile.
    ///
    /// ## Errors
    ///
    /// Returns an IO error if the csv reader is unable to be built, or a line
    /// of the csv reader is unable to be parsed, or if the quantile is unable
    /// to be calculated because `all_alleles_data.totals` is empty
    pub fn import_from_file(filename: &Path) -> Result<Self, PlotError> {
        let mut all_alleles_data = AllAlleles {
            totals: Totals {
                data: Vec::new(),
                upper_quantile: 0.0,
            },
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
            .from_path(filename)
            .map_err(|err| {
                PlotError::IOError(format!("opening '{}'", filename.display()), err.into())
            })?;

        for line in all_alleles_reader.deserialize() {
            let line: AllAllelesLine =
                line.map_err(|err| PlotError::InvalidData(err.to_string()))?;

            all_alleles_data.totals.data.push(line.total);

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

            if let Some(conf) = line.confidence_not_mac_err
                && conf > 0.0
            {
                all_alleles_data.confidence_not_mac_errs.push(conf);
            }
        }

        let upper_quantile =
            quantile(&all_alleles_data.totals.data, TOTAL_PROB).map_err(|err| {
                PlotError::InvalidData(format!(
                    "Error calculating totals quantile from {}: {}",
                    filename.display(),
                    err
                ))
            })?;
        all_alleles_data.totals.data = all_alleles_data
            .totals
            .data
            .into_iter()
            .filter(|x| *x <= upper_quantile)
            .collect::<Vec<_>>();
        all_alleles_data.totals.upper_quantile = upper_quantile;

        Ok(all_alleles_data)
    }
}

pub struct AverageQualities {
    pub data: Vec<f64>,
    pub min: f64,
    pub max: f64,
}

pub struct Totals {
    pub data: Vec<f64>,
    pub upper_quantile: f64,
}

/// The quantile of observations `x` at probability `p`. Assumes all
/// observations `x` have equal weight. Eurostat definition.
///
/// ## Errors
///
/// Returns an error if the slice of observations is empty
fn quantile(observations: &[f64], probability: f64) -> Result<f64, PlotError> {
    if observations.is_empty() {
        return Err(PlotError::MissingData(
            "Allele observations must not be empty".to_string(),
        ));
    }
    // this is checking a const, it won't ever be changed by a user
    if !(0.0..=1.0).contains(&probability) {
        return Err(PlotError::InvalidData(
            "const TOTAL_PROB must be between 0.0 and 1.0".to_string(),
        ));
    }

    let mut sorted = observations.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let n = sorted.len() as f64;
    let pos = probability * (n - 1.0);
    let lower_idx = pos.floor() as usize;
    let upper_idx = pos.ceil() as usize;

    Ok(if lower_idx == upper_idx {
        sorted[lower_idx]
    } else {
        let t = pos - lower_idx as f64;
        sorted[lower_idx] + t * (sorted[upper_idx] - sorted[lower_idx])
    })
}
