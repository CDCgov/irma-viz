use std::{
    fs::File,
    io::{BufReader, prelude::*},
};
use zoe::data::err::ResultWithErrorContext;

// Header format for `<gene>-allAlleles.txt`
const ALL_ALLELES_HEADER: &str = "Reference_Name\tPosition\tAllele\tCount\tTotal\tFrequency\tAverage_Quality\tConfidenceNotMacErr\tPairedUB\tQualityUB\tAllele_Type";
const REF_NAME_COL: usize = 0;
const POS_COL: usize = 1;
const ALL_COL: usize = 2;
const CNT_COL: usize = 3;
const TOT_COL: usize = 4;
const FREQ_COL: usize = 5;
const AQ_COL: usize = 6;
const CNME_COL: usize = 7;
const PUB_COL: usize = 8;
const QUB_COL: usize = 9;
const TYPE_COL: usize = 10;
const MAX_COLS: usize = 11;

enum AlleleType {
    Consensus,
    Minority,
}

struct AllAllelesLine {
    reference_name: String,
    position: usize,
    allele: u8,
    count: usize,
    total: usize,
    frequency: f64,
    average_quality: f64,
    confidence_not_mac_err: f64,
    paired_ub: f64,
    quality_ub: f64,
    allele_type: AlleleType,
}

struct AllAllelesData {
    reference_name: String,
    positions: Vec<usize>,
    alleles: Vec<u8>,
    counts: Vec<usize>,
    totals: Vec<usize>,
    frequencies: Vec<f64>,
    average_qualities: Vec<f64>,
    confidence_not_mac_errs: Vec<f64>,
    paried_ubs: Vec<f64>,
    quality_ubs: Vec<f64>,
    allele_types: Vec<AlleleType>,
}

impl AllAllelesData {
    fn import_from_file(filename: &str) -> std::io::Result<Self> {
        let mut all_alleles_lines =
            BufReader::new(File::open(filename).with_context("Cannot open file")?).lines();

        let Some(all_alleles_header) = all_alleles_lines.next().transpose()? else {
            return Err(std::io::Error::other("File is empty."));
        };
        if all_alleles_header != ALL_ALLELES_HEADER {
            return Err(std::io::Error::other("Invalid header format."));
        }

        let Some(first_all_alleles_line_str) = all_alleles_lines.next().transpose()? else {
            return Err(std::io::Error::other("File has no data."));
        };
        let first_all_alleles_line = Self::parse_line(first_all_alleles_line_str)
            .with_context("Failed to parse line number 1")?;

        let mut all_alleles_data = AllAllelesData {
            reference_name: first_all_alleles_line.reference_name,
            positions: vec![first_all_alleles_line.position],
            alleles: vec![first_all_alleles_line.allele],
            counts: vec![first_all_alleles_line.count],
            totals: vec![first_all_alleles_line.total],
            frequencies: vec![first_all_alleles_line.frequency],
            average_qualities: vec![first_all_alleles_line.average_quality],
            confidence_not_mac_errs: vec![first_all_alleles_line.confidence_not_mac_err],
            paried_ubs: vec![first_all_alleles_line.paired_ub],
            quality_ubs: vec![first_all_alleles_line.quality_ub],
            allele_types: vec![first_all_alleles_line.allele_type],
        };
        for (line_num, line) in all_alleles_lines.enumerate() {
            let line = line?;

            if line.is_empty() {
                continue;
            }

            let all_alleles_line = Self::parse_line(line)
                .with_context(format!("Failed to parse line number {}", line_num + 2))?;

            if all_alleles_data.reference_name != all_alleles_line.reference_name {
                return Err(std::io::Error::other(format!(
                    "Reference name has changed on line number {}",
                    line_num + 2
                )));
            }
        }

        Ok(all_alleles_data)
    }

    fn parse_line(line: String) -> std::io::Result<AllAllelesLine> {
        let split_line = line.split('\t').collect::<Vec<_>>();
        if split_line.len() < MAX_COLS {
            return Err(std::io::Error::other(format!(
                "Too few fields. Found: {found}, expected: {MAX_COLS}",
                found = split_line.len()
            )));
        }

        let reference_name = split_line[REF_NAME_COL].to_string();
        let position = split_line[POS_COL]
            .parse::<usize>()
            .with_context("Failed to parse Position field.")?;
        let allele = match split_line[ALL_COL] {
            "A" | "C" | "G" | "T" => split_line[ALL_COL].as_bytes()[0],
            _ => {
                return Err(std::io::Error::other(
                    "Failed to parse Allele field. Allele is not \"A\", \"C\", \"G\", or \"T\".",
                ));
            }
        };
        let count = split_line[POS_COL]
            .parse::<usize>()
            .with_context("Failed to parse Count field.")?;
        let total = split_line[TOT_COL]
            .parse::<usize>()
            .with_context("Failed to parse Total field.")?;
        let frequency = split_line[FREQ_COL]
            .parse::<f64>()
            .with_context("Failed to parse Frequency field.")?;
        let average_quality = split_line[AQ_COL]
            .parse::<f64>()
            .with_context("Failed to parse Average_Quality field.")?;
        let confidence_not_mac_err = split_line[CNME_COL]
            .parse::<f64>()
            .with_context("Failed to parse ConfidenceNotMacErr field.")?;
        let paired_ub = split_line[PUB_COL]
            .parse::<f64>()
            .with_context("Failed to parse PairedUB field")?;
        let quality_ub = split_line[QUB_COL]
            .parse::<f64>()
            .with_context("Failed to parse QualityUB field.")?;
        let allele_type = match split_line[TYPE_COL] {
            "Consensus" => AlleleType::Consensus,
            "Minority" => AlleleType::Minority,
            _ => {
                return Err(std::io::Error::other(
                    "Failed to parse Allele_Type field. Allele_Type is not \"Consensus\" or \"Minority\".",
                ));
            }
        };

        Ok(AllAllelesLine {
            reference_name,
            position,
            allele,
            count,
            total,
            frequency,
            average_quality,
            confidence_not_mac_err,
            paired_ub,
            quality_ub,
            allele_type,
        })
    }
}
