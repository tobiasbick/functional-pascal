use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const INVALID_THRESHOLD: &str = "error: --threshold-pct must be finite and non-negative";

fn run_with_threshold(value: &str) -> Result<(Option<i32>, String), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-bench"))
        .args(["compare", "baseline", "--threshold-pct", value])
        .output()?;

    Ok((
        output.status.code(),
        String::from_utf8(output.stderr)?.trim().to_owned(),
    ))
}

#[test]
fn nan_threshold_exits_with_actionable_error() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        run_with_threshold("NaN")?,
        (Some(1), INVALID_THRESHOLD.to_owned())
    );
    Ok(())
}

#[test]
fn positive_infinite_threshold_exits_with_actionable_error() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        run_with_threshold("inf")?,
        (Some(1), INVALID_THRESHOLD.to_owned())
    );
    Ok(())
}

#[test]
fn negative_infinite_threshold_exits_with_actionable_error() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        run_with_threshold("-inf")?,
        (Some(1), INVALID_THRESHOLD.to_owned())
    );
    Ok(())
}

#[test]
fn negative_finite_threshold_exits_with_actionable_error() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        run_with_threshold("-0.5")?,
        (Some(1), INVALID_THRESHOLD.to_owned())
    );
    Ok(())
}

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn with_mismatched_group_snapshot() -> io::Result<Self> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let path = loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let candidate =
                std::env::temp_dir().join(format!("fpas-bench-cli-{}-{id}", std::process::id()));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        };

        let repo = Self { path };
        fs::create_dir_all(repo.path.join("docs/bench"))?;
        fs::create_dir_all(repo.path.join(".temp-data/bench"))?;
        fs::write(
            repo.path.join("docs/bench/suite.toml"),
            "[[bench]]\nid = \"tui_headless\"\ngroup = \"tui\"\npath = \"unused.fpas\"\nargs = []\ntimeout_ms = 1000\n",
        )?;
        fs::write(
            repo.path.join(".temp-data/bench/baseline.json"),
            "{\n  \"label\": \"baseline\",\n  \"timestamp_unix\": 0,\n  \"group\": \"vm\",\n  \"benches\": []\n}\n",
        )?;
        Ok(repo)
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn mismatched_snapshot_group_fails_before_release_setup() -> Result<(), Box<dyn Error>> {
    let repo = TempRepo::with_mismatched_group_snapshot()?;
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-bench"))
        .args(["compare", "baseline", "--group", "tui"])
        .current_dir(repo.path())
        .env_remove("CARGO_MANIFEST_DIR")
        .output()?;
    let stderr = String::from_utf8(output.stderr)?.trim().to_owned();

    assert_eq!(
        (output.status.code(), stderr, repo.path().join("target").exists()),
        (
            Some(1),
            "error: baseline snapshot `baseline` was saved for group `vm`, but compare requested `tui`; replace it with `cargo bench-fpas save baseline --group tui`".to_owned(),
            false
        )
    );
    Ok(())
}

#[test]
fn help_exits_successfully_and_lists_every_group() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-bench"))
        .arg("--help")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    assert_eq!(
        (
            output.status.code(),
            stderr,
            stdout.contains("--group vm|concurrency|tui"),
            stdout.contains("Examples:")
        ),
        (Some(0), String::new(), true, true)
    );
    Ok(())
}

#[test]
fn unknown_group_lists_every_configured_group() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_fpas-bench"))
        .args(["run", "--group", "missing"])
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?.trim().to_owned();

    assert_eq!(
        (output.status.code(), stdout, stderr),
        (
            Some(1),
            String::new(),
            "error: no benchmarks in group `missing` (known groups: vm, concurrency, tui)"
                .to_owned()
        )
    );
    Ok(())
}
