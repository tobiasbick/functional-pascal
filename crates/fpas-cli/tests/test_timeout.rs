//! Integration tests for enforcing FPAS test-process timeouts.

#![allow(
    clippy::expect_used,
    reason = "timeout integration fixtures require direct process and filesystem assertions"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let path = std::env::temp_dir().join(format!(
        "fpas-timeout-integration-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).expect("temporary directory must be created");
    path
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).expect("fixture must be written");
}

fn run_timed_test(root: &Path, test_file: &Path) -> std::process::Output {
    let fpas = Path::new(env!("CARGO_BIN_EXE_fpas"));
    let standard_library = fpas.parent().expect("binary directory").join("lib");
    let scratch = root.join("scratch");
    fs::create_dir(&scratch).expect("scratch directory");

    Command::new(fpas)
        .current_dir(root)
        .args(["test", "--timeout", "1", "--std-lib"])
        .arg(standard_library)
        .arg(test_file)
        .env("TMP", &scratch)
        .env("TEMP", &scratch)
        .env("TMPDIR", &scratch)
        .output()
        .expect("fpas test must start")
}

#[test]
fn timeout_force_stops_blocking_sleep_and_cleans_worker_files() {
    let root = temp_dir();
    let test_file = root.join("sleep_test.fpas");
    write(
        &test_file,
        "program SleepTest;\nuses Std.Time;\nbegin Sleep(60000) end.",
    );

    let started = Instant::now();
    let output = run_timed_test(&root, &test_file);

    assert_eq!(output.status.code(), Some(3));
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "non-cooperative timeout took {:?}",
        started.elapsed()
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("TIMEOUT  sleep_test.fpas"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_dir(root.join("scratch"))
            .expect("scratch directory")
            .count(),
        0,
        "isolated worker files must be removed"
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn timeout_terminates_processes_started_by_the_test() {
    let root = temp_dir();
    let test_file = root.join("process_test.fpas");
    let sentinel = root.join("descendant-survived.txt");
    let (command, arguments) = descendant_command(&sentinel);
    write(
        &test_file,
        &format!(
            "program ProcessTest;\nuses Std.Proc;\nmutable var Status: Result of integer, string := Error('not started');\nbegin Status := Run('{command}', [{arguments}]) end."
        ),
    );

    let output = run_timed_test(&root, &test_file);
    assert_eq!(
        output.status.code(),
        Some(3),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    thread::sleep(Duration::from_secs(4));
    assert!(
        !sentinel.exists(),
        "a descendant process survived the timed test worker"
    );
    fs::remove_dir_all(root).ok();
}

#[cfg(windows)]
fn descendant_command(sentinel: &Path) -> (&'static str, String) {
    (
        "cmd",
        format!(
            "'/C', 'ping -n 4 127.0.0.1 > nul & echo survived > \"{}\"'",
            sentinel.display()
        ),
    )
}

#[cfg(unix)]
fn descendant_command(sentinel: &Path) -> (&'static str, String) {
    (
        "sh",
        format!(
            "'-c', 'sleep 3; echo survived > \"{}\"'",
            sentinel.display()
        ),
    )
}
