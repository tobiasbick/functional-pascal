//! Curated FPAS end-to-end benchmark harness.
//!
//! See [`docs/bench/README.md`](../../../docs/bench/README.md).

mod results;
mod suite;

use results::{
    CompareRow, compare_runs, has_regression, load_snapshot, record_history, save_snapshot,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use suite::{
    BenchRun, ensure_release_fpas, filter_group, load_suite, run_suite, unix_timestamp_secs,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = parse_args(&args)?;
    let repo_root = find_repo_root()?;
    let suite = load_suite(&repo_root)?;
    let specs = filter_group(&suite, options.group.as_deref())?;
    let fpas = ensure_release_fpas(&repo_root)?;

    match options.command {
        Command::Run => {
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            print_run_table(&runs);
            Ok(ExitCode::SUCCESS)
        }
        Command::Save { label } => {
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            print_run_table(&runs);
            let path = save_snapshot(&repo_root, &label, unix_timestamp_secs(), &runs)?;
            println!("saved {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Compare { label } => {
            let baseline = load_snapshot(&repo_root, &label)?;
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            let rows = compare_runs(&baseline, &runs);
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
            let runs = run_suite(&repo_root, &fpas, &specs)?;
            print_run_table(&runs);
            let path = record_history(&repo_root, &title, options.group.as_deref(), &runs)?;
            println!("recorded {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[derive(Debug)]
enum Command {
    Run,
    Save { label: String },
    Compare { label: String },
    Record { title: String },
}

#[derive(Debug)]
struct Options {
    command: Command,
    group: Option<String>,
    fail_on_regression: bool,
    threshold_pct: f64,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    if args.is_empty() {
        return Err(usage());
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
                    .ok_or_else(|| "missing value for --group".to_owned())?;
                group = Some(value.clone());
            }
            "--fail-on-regression" => {
                fail_on_regression = true;
            }
            "--threshold-pct" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "missing value for --threshold-pct".to_owned())?;
                threshold_pct = value
                    .parse::<f64>()
                    .map_err(|_| format!("invalid --threshold-pct value: {value}"))?;
                if threshold_pct < 0.0 {
                    return Err("--threshold-pct must be non-negative".to_owned());
                }
            }
            "--help" | "-h" => return Err(usage()),
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag `{flag}`\n{}", usage()));
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
        _ => return Err(usage()),
    };

    Ok(Options {
        command,
        group,
        fail_on_regression,
        threshold_pct,
    })
}

fn usage() -> String {
    "usage:\n  fpas-bench run [--group vm|tui]\n  fpas-bench save <label> [--group vm|tui]\n  fpas-bench compare <label> [--group vm|tui] [--fail-on-regression] [--threshold-pct N]\n  fpas-bench record <title…> [--group vm|tui]\n\nSee docs/bench/README.md.".to_owned()
}

fn find_repo_root() -> Result<PathBuf, String> {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let crate_dir = PathBuf::from(manifest_dir);
        if let Some(root) = crate_dir.parent().and_then(Path::parent) {
            if root.join("docs/bench/suite.toml").is_file() {
                return Ok(root.to_path_buf());
            }
        }
    }

    let mut dir = env::current_dir().map_err(|error| format!("cwd: {error}"))?;
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
    println!("{:<16} {:>10}  {}", "bench", "elapsed_ms", "throughput");
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
        "{:<16} {:>10} {:>10} {:>10}  {}",
        "bench", "before_ms", "after_ms", "delta_%", "throughput"
    );
    println!("{}", "-".repeat(72));
    for row in rows {
        let before = row
            .baseline_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_owned());
        let delta = row
            .delta_pct
            .map(|value| format!("{value:+.1}"))
            .unwrap_or_else(|| "-".to_owned());
        let throughput = row.throughput.as_deref().unwrap_or("-");
        println!(
            "{:<16} {:>10} {:>10} {:>10}  {}",
            row.id, before, row.current_ms, delta, throughput
        );
    }
    println!();
}
