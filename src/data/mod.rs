use serde::{Deserialize, de::Error};

pub mod all_alleles;
pub mod coverage;
pub mod pairing_stats;
pub mod read_counts;
pub mod square_matrix;
pub mod variants;

pub use all_alleles::*;
pub use coverage::*;
pub use pairing_stats::*;
pub use read_counts::*;
pub use square_matrix::*;
pub use variants::*;

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
fn allele_char<'de, D>(deserializer: D) -> Result<char, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    if let &[s] = s
        .to_ascii_uppercase()
        .chars()
        .collect::<Vec<_>>()
        .as_slice()
    {
        match s {
            'A' | 'C' | 'G' | 'T' => Ok(s),
            'N' | '-' | '.' => Ok('N'),
            _ => Err(D::Error::custom(
                "Failed to parse Allele field. Allele is not \"A\", \"C\", \"G\", \"T\", \"N\", or \"-\".",
            )),
        }
    } else {
        Err(D::Error::custom(
            "Faield to parse Allele field. Allele is not a single character.",
        ))
    }
}
