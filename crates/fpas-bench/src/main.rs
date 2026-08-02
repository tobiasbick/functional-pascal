//! Curated FPAS end-to-end benchmark harness.
//!
//! See [`docs/bench/README.md`](../../../docs/bench/README.md).

mod arguments;
mod results;
mod suite;

use arguments::{Command, Options, ParseError, ParseOutcome, parse_args, usage};
use results::{
    CompareRow, ComparisonBaseline, has_regression, load_snapshot, record_history, save_snapshot,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use suite::{
    BenchRun, ensure_release_fpas, filter_group, group_names, load_suite, run_suite,
    unix_timestamp_secs,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&args) {
        Ok(ParseOutcome::Help) => {
            println!("{}", usage(&configured_group_names()));
            ExitCode::SUCCESS
        }
        Ok(ParseOutcome::Execute(options)) => finish(execute(options)),
        Err(ParseError::Message(message)) => fail(&message, false),
        Err(ParseError::Usage(message)) => fail(&message, true),
    }
}

fn finish(result: Result<ExitCode, String>) -> ExitCode {
    match result {
        Ok(code) => code,
        Err(message) => fail(&message, false),
    }
}

fn fail(message: &str, show_usage: bool) -> ExitCode {
    eprintln!("error: {message}");
    if show_usage {
        eprintln!("\n{}", usage(&configured_group_names()));
    }
    ExitCode::from(1)
}

fn configured_group_names() -> Vec<String> {
    find_repo_root()
        .and_then(|repo_root| load_suite(&repo_root))
        .map(|suite| group_names(&suite))
        .unwrap_or_default()
}

fn execute(options: Options) -> Result<ExitCode, String> {
    let repo_root = find_repo_root()?;
    let suite = load_suite(&repo_root)?;
    let specs = filter_group(&suite, options.group.as_deref())?;

    match options.command {
        Command::Run => {
            let fpas = ensure_release_fpas(&repo_root)?;
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            print_run_table(&runs);
            Ok(ExitCode::SUCCESS)
        }
        Command::Save { label } => {
            let fpas = ensure_release_fpas(&repo_root)?;
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            print_run_table(&runs);
            let path = save_snapshot(
                &repo_root,
                &label,
                unix_timestamp_secs(),
                options.group.as_deref(),
                &runs,
            )?;
            println!("saved {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Compare { label } => {
            let baseline = load_snapshot(&repo_root, &label)?;
            let baseline = ComparisonBaseline::new(&baseline, options.group.as_deref())?;
            let fpas = ensure_release_fpas(&repo_root)?;
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            let rows = baseline.compare(&runs)?;
            print_compare_table(&rows);
            if options.fail_on_regression && has_regression(&rows, options.threshold_pct) {
                eprintln!(
                    "regression: at least one bench slowed by more than {:.1}%",
                    options.threshold_pct
                );
                return Ok(ExitCode::from(2));
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Record { title } => {
            let fpas = ensure_release_fpas(&repo_root)?;
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            print_run_table(&runs);
            let path = record_history(&repo_root, &title, options.group.as_deref(), &runs)?;
            println!("recorded {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn find_repo_root() -> Result<PathBuf, String> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let crate_dir = PathBuf::from(manifest_dir);
        if let Some(root) = crate_dir.parent().and_then(Path::parent)
            && root.join("docs/bench/suite.toml").is_file()
        {
            return Ok(root.to_path_buf());
        }
    }

    let mut dir = std::env::current_dir().map_err(|error| format!("cwd: {error}"))?;
    loop {
        if dir.join("docs/bench/suite.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("could not find repository root containing docs/bench/suite.toml".to_owned())
}

fn print_run_table(runs: &[BenchRun]) {
    println!();
    println!("{:<16} {:>10}  throughput", "bench", "elapsed_ms");
    println!("{}", "-".repeat(60));
    for run in runs {
        let throughput = run.throughput.as_deref().unwrap_or("-");
        println!("{:<16} {:>10}  {}", run.id, run.elapsed_ms, throughput);
    }
    println!();
}

fn print_compare_table(rows: &[CompareRow]) {
    println!();
    println!(
        "{:<16} {:>10} {:>10} {:>10}  throughput",
        "bench", "before_ms", "after_ms", "delta_%"
    );
    println!("{}", "-".repeat(72));
    for row in rows {
        let before = row.baseline_ms.to_string();
        let delta = format!("{:+.1}", row.delta_pct);
        let throughput = row.throughput.as_deref().unwrap_or("-");
        println!(
            "{:<16} {:>10} {:>10} {:>10}  {}",
            row.id, before, row.current_ms, delta, throughput
        );
    }
    println!();
}
