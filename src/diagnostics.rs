use jiff::Zoned;
use std::fmt::Display;

const PROGRAM: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Success,
    Failure,
}

impl Severity {
    fn as_label(self) -> &'static str {
        match self {
            Severity::Warning => "WARNING",
            Severity::Success => "SUCCESS",
            Severity::Failure => "FAILURE",
        }
    }
}

fn format_warning(severity: Severity, message: impl Display) -> String {
    let shlvl = std::env::var("SHLVL")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let pad = "  ".repeat(shlvl.saturating_sub(1));

    format!(
        "[{now}] {pad}{PROGRAM} {} :: {message}",
        severity.as_label(),
        now = Zoned::now().strftime("%Y-%m-%d %k:%M:%S")
    )
}

/// Formats error output into parseable log
pub fn warn(severity: Severity, message: impl Display) {
    eprintln!("{}", format_warning(severity, message));
}

#[derive(Debug)]
pub enum PlotError {
    MissingData(String),
    InvalidData(String),
    RenderError(String),
    IOError(String, std::io::Error),
    ConfigError(String),
}

impl Display for PlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotError::MissingData(err) => write!(f, "{err}"),
            PlotError::RenderError(err) => write!(f, "{err}"),
            PlotError::IOError(context, err) => write!(f, "{context}: {err}"),
            PlotError::ConfigError(err) => write!(f, "{err}"),
            PlotError::InvalidData(err) => write!(f, "{err}"),
        }
    }
}

pub fn print_results<I>(names: I, plot_type: &str)
where
    I: IntoIterator<Item = String>,
{
    let names = names.into_iter().collect::<Vec<_>>();

    if names.is_empty() {
        warn(
            Severity::Warning,
            format!("Unable to create any valid {plot_type} plots"),
        );
    } else {
        warn(
            Severity::Success,
            format!(
                "Created {} {plot_type} plots: {}",
                names.len(),
                names.join(", ")
            ),
        )
    }
}
