pub mod all_alleles;
pub mod read_counts;
pub use all_alleles::*;
pub use read_counts::*;
use serde::{Deserialize, de::Error};

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
