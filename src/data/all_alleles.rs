use crate::data::*;
use anyhow::{Context, Result, anyhow};
use std::path::Path;

const TOTAL_PROB: f64 = 0.2;

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
    pub confidence_not_mac_errs: Vec<f64>,
}

impl AllAlleles {
    /// TODO: Docs
    pub fn import_from_file(filename: &Path) -> Result<Self> {
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
            .from_path(filename)
            .with_context(|| format!("Cannot read {}", filename.display()))?;

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

            if let Some(conf) = line.confidence_not_mac_err
                && conf > 0.0
            {
                all_alleles_data.confidence_not_mac_errs.push(conf);
            }
        }

        let mx = quantile(&all_alleles_data.totals, TOTAL_PROB).with_context(|| {
            format!(
                "Error calculating totals quantile from {}",
                filename.display(),
            )
        })?;
        all_alleles_data.totals = all_alleles_data
            .totals
            .into_iter()
            .filter(|x| *x < mx)
            .collect::<Vec<_>>();

        Ok(all_alleles_data)
    }
}

pub struct AverageQualities {
    pub data: Vec<f64>,
    pub min: f64,
    pub max: f64,
}

/// The quantile of observations `x` at probability `p`. Assumes all
/// observations `x` have equal weight. Eurostat definition.
fn quantile(x: &[f64], p: f64) -> Result<f64> {
    if x.is_empty() {
        return Err(anyhow!("Observations must not be empty"));
    }
    if !(0.0..=1.0).contains(&p) {
        return Err(anyhow!("Probability must be between 0.0 and 1.0"));
    }

    let mut sorted = x.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let n = sorted.len() as f64;
    let pos = p * (n - 1.0);
    let lower_idx = pos.floor() as usize;
    let upper_idx = pos.ceil() as usize;

    Ok(if lower_idx == upper_idx {
        sorted[lower_idx]
    } else {
        let t = pos - lower_idx as f64;
        sorted[lower_idx] + t * (sorted[upper_idx] - sorted[lower_idx])
    })
}
