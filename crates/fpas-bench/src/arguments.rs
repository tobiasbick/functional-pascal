//! Parse benchmark harness arguments and render focused command help.

/// Parsed command that executes benchmark work.
#[derive(Debug)]
pub enum Command {
    /// Run selected benchmarks without persistence.
    Run,
    /// Run and save a labeled baseline snapshot.
    Save { label: String },
    /// Run and compare with a labeled baseline snapshot.
    Compare { label: String },
    /// Run and record results in committed history.
    Record { title: String },
}

/// Validated command-line options.
#[derive(Debug)]
pub struct Options {
    /// Requested operation.
    pub command: Command,
    /// Optional suite group filter.
    pub group: Option<String>,
    /// Whether a detected regression returns exit code 2.
    pub fail_on_regression: bool,
    /// Allowed slowdown percentage for regression gating.
    pub threshold_pct: f64,
}

/// Successful parser result.
#[derive(Debug)]
pub enum ParseOutcome {
    /// Print help and exit successfully.
    Help,
    /// Execute a validated command.
    Execute(Options),
}

/// Actionable argument parsing failure.
#[derive(Debug)]
pub enum ParseError {
    /// Print only the focused error message.
    Message(String),
    /// Print the error followed by usage and examples.
    Usage(String),
}

/// Parse raw arguments without performing repository or benchmark work.
pub fn parse_args(args: &[String]) -> Result<ParseOutcome, ParseError> {
    if args.is_empty() {
        return Err(ParseError::Usage("missing command".to_owned()));
    }

    let mut group = None;
    let mut fail_on_regression = false;
    let mut threshold_pct = 10.0_f64;
    let mut positional = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--group" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| ParseError::Message("missing value for --group".to_owned()))?;
                group = Some(value.clone());
            }
            "--fail-on-regression" => {
                fail_on_regression = true;
            }
            "--threshold-pct" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    ParseError::Message("missing value for --threshold-pct".to_owned())
                })?;
                threshold_pct = value.parse::<f64>().map_err(|_| {
                    ParseError::Message(format!("invalid --threshold-pct value: {value}"))
                })?;
                if !threshold_pct.is_finite() || threshold_pct < 0.0 {
                    return Err(ParseError::Message(
                        "--threshold-pct must be finite and non-negative".to_owned(),
                    ));
                }
            }
            "--help" | "-h" => return Ok(ParseOutcome::Help),
            flag if flag.starts_with('-') => {
                return Err(ParseError::Usage(format!("unknown flag `{flag}`")));
            }
            other => positional.push(other.to_owned()),
        }
        index += 1;
    }

    let command = match positional.first().map(String::as_str) {
        Some("run") if positional.len() == 1 => Command::Run,
        Some("save") if positional.len() == 2 => Command::Save {
            label: positional[1].clone(),
        },
        Some("compare") if positional.len() == 2 => Command::Compare {
            label: positional[1].clone(),
        },
        Some("record") if positional.len() >= 2 => Command::Record {
            title: positional[1..].join(" "),
        },
        _ => {
            return Err(ParseError::Usage("invalid command or arguments".to_owned()));
        }
    };

    Ok(ParseOutcome::Execute(Options {
        command,
        group,
        fail_on_regression,
        threshold_pct,
    }))
}

/// Render usage with group names derived from the loaded suite.
pub fn usage(groups: &[String]) -> String {
    let groups = if groups.is_empty() {
        "<group>".to_owned()
    } else {
        groups.join("|")
    };
    format!(
        "Usage:\n  cargo bench-fpas run [--group {groups}]\n  cargo bench-fpas save <label> [--group {groups}]\n  cargo bench-fpas compare <label> [--group {groups}] [--fail-on-regression] [--threshold-pct N]\n  cargo bench-fpas record <title…> [--group {groups}]\n  cargo bench-fpas native --help\n\nExamples:\n  cargo bench-fpas run --group vm\n  cargo bench-fpas save before --group concurrency\n  cargo bench-fpas compare before --group vm --fail-on-regression --threshold-pct 10\n  cargo bench-fpas record \"after runtime change\" --group tui\n\nSee docs/bench/README.md."
    )
}

#[cfg(test)]
mod tests {
    use super::{ParseError, ParseOutcome, parse_args, usage};

    const INVALID_THRESHOLD: &str = "--threshold-pct must be finite and non-negative";

    fn parse_threshold(value: &str) -> Result<f64, String> {
        let args = ["compare", "baseline", "--threshold-pct", value].map(str::to_owned);
        match parse_args(&args) {
            Ok(ParseOutcome::Execute(options)) => Ok(options.threshold_pct),
            Ok(ParseOutcome::Help) => Err("unexpected help".to_owned()),
            Err(ParseError::Message(message) | ParseError::Usage(message)) => Err(message),
        }
    }

    #[test]
    fn help_is_a_successful_parser_outcome() {
        assert!(matches!(
            parse_args(&["--help".to_owned()]),
            Ok(ParseOutcome::Help)
        ));
    }

    #[test]
    fn usage_includes_all_loaded_groups() {
        let groups = ["vm", "concurrency", "tui"].map(str::to_owned);
        assert!(usage(&groups).contains("--group vm|concurrency|tui"));
    }

    #[test]
    fn threshold_rejects_nan() {
        assert_eq!(parse_threshold("NaN"), Err(INVALID_THRESHOLD.to_owned()));
    }

    #[test]
    fn threshold_rejects_positive_infinity() {
        assert_eq!(parse_threshold("inf"), Err(INVALID_THRESHOLD.to_owned()));
    }

    #[test]
    fn threshold_rejects_negative_infinity() {
        assert_eq!(parse_threshold("-inf"), Err(INVALID_THRESHOLD.to_owned()));
    }

    #[test]
    fn threshold_rejects_negative_finite_value() {
        assert_eq!(parse_threshold("-0.5"), Err(INVALID_THRESHOLD.to_owned()));
    }

    #[test]
    fn threshold_accepts_zero() {
        assert_eq!(parse_threshold("0"), Ok(0.0));
    }

    #[test]
    fn threshold_accepts_normal_value() {
        assert_eq!(parse_threshold("12.5"), Ok(12.5));
    }
}
