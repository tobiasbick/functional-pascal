//! Native tooling workloads run in the harness's bounded child processes.

mod compiler;
mod language_service;

use std::process::ExitCode;

const USAGE: &str = "Usage:\n  cargo bench-fpas native language-service <queries> <functions>\n  cargo bench-fpas native compiler-lowering <iterations> <branches>\n\nExamples:\n  cargo bench-fpas native language-service 1000 500\n  cargo bench-fpas native compiler-lowering 30 1000\n  cargo bench-fpas save tooling-before --group tooling";

/// Executes one native workload, or prints its usage.
pub(crate) fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    let [driver, iterations, width] = args else {
        return Err(USAGE.to_owned());
    };
    match driver.as_str() {
        "language-service" => {
            language_service::run(positive_count(iterations)?, positive_count(width)?)?
        }
        "compiler-lowering" => compiler::run(positive_count(iterations)?, positive_count(width)?)?,
        _ => return Err(format!("Unknown native workload `{driver}`.\n{USAGE}")),
    }
    Ok(ExitCode::SUCCESS)
}

fn positive_count(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("Expected a positive workload count, received `{value}`.\n{USAGE}"))
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn invalid_workloads_fail_before_running() {
        for args in [
            vec![],
            vec!["missing", "1", "1"],
            vec!["language-service", "0", "1"],
            vec!["language-service", "1", "-1"],
        ] {
            assert!(run(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err());
        }
    }
}
