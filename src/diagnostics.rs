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

/// Enum with arms for different error types that could occur throughout the
/// data gathering and plotting process
///
/// Most variants currently just wrap a `String`, so these arms are effectively
/// interchangeable. This could be changed to actually handle the error type, or
/// we could unify them later on
///
/// Since they only hold strings, wrapping one error within another allows us to
/// have nested levels of context
#[derive(Debug)]
pub enum PlotError {
    /// Required input was missing or present but empty.
    MissingData(String),
    /// Input was present but malformed or otherwise could not be parsed.
    InvalidData(String),
    /// An IO operation failed. The first string stores caller-provided context
    /// that is prepended to the underlying IO error message.
    IOError(String, std::io::Error),
    /// The provided configuration was invalid or internally inconsistent.
    ConfigError(String),
    /// PDF render error
    #[cfg(feature = "pdf")]
    RenderError(String),
}

impl Display for PlotError {
    /// Formats errors for display. Note, as mentioned [`PlotError`]'s arms are
    /// effectively the same except for [`PlotError::IOError`], so are handled
    /// the same as well
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotError::MissingData(err) => write!(f, "{err}"),
            PlotError::IOError(context, err) => write!(f, "{context}: {err}"),
            PlotError::ConfigError(err) => write!(f, "{err}"),
            PlotError::InvalidData(err) => write!(f, "{err}"),
            #[cfg(feature = "pdf")]
            PlotError::RenderError(err) => write!(f, "{err}"),
        }
    }
}

/// Warns that a specific plot (with optional target) is being skipped because
/// of the provided error.
pub fn warn_plot_error(plot_type: &str, target: Option<&str>, err: &PlotError) {
    match target {
        Some(target) => warn(
            Severity::Warning,
            format!("skipping {plot_type} plot for '{target}': {err}"),
        ),
        None => warn(
            Severity::Warning,
            format!("skipping {plot_type} plot: {err}"),
        ),
    }
}

/// Prints results to std error with a Success and the number of plots created
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
