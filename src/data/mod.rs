pub mod all_alleles;
pub mod coverage;
pub mod read_counts;
pub use all_alleles::*;
pub use coverage::*;
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
