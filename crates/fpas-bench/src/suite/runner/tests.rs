//! Process lifetime and timeout regressions.

use super::{parse_elapsed_ms, spawn_benchmark, wait_for_output};
use std::error::Error;
use std::fs;
use std::io;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn parse_elapsed_ms_reads_standard_line() {
    let stdout = "iterations: 1\nelapsed: 142 ms\nthroughput: 7 ops/s\n";
    assert_eq!(parse_elapsed_ms(stdout), Some(142));
}

#[test]
fn parse_elapsed_ms_ignores_noise() {
    assert_eq!(parse_elapsed_ms("no timing here\n"), None);
}

#[test]
fn timeout_child_fixture() {
    if std::env::var_os("FPAS_BENCH_TIMEOUT_FIXTURE").is_none() {
        return;
    }
    println!("child started");
    eprintln!("child waiting");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn timeout_tree_parent_fixture() -> Result<(), Box<dyn Error>> {
    let Some(ready_path) = std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_READY") else {
        return Ok(());
    };
    let executable = std::env::current_exe()?;
    Command::new(executable)
        .args([
            "--exact",
            "suite::runner::tests::timeout_tree_grandchild_fixture",
            "--nocapture",
        ])
        .env("FPAS_BENCH_TIMEOUT_TREE_GRANDCHILD", "1")
        .env("FPAS_BENCH_TIMEOUT_TREE_READY", ready_path)
        .spawn()?;
    if std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_PARENT_EXIT").is_some() {
        return Ok(());
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn timeout_tree_grandchild_fixture() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_GRANDCHILD").is_none() {
        return Ok(());
    }
    println!("grandchild started");
    let ready_path = std::env::var_os("FPAS_BENCH_TIMEOUT_TREE_READY")
        .ok_or_else(|| io::Error::other("missing timeout tree ready path"))?;
    fs::write(ready_path, [])?;
    let hold = std::env::var("FPAS_BENCH_TIMEOUT_TREE_HOLD")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(2);
    std::thread::sleep(Duration::from_secs(hold));
    Ok(())
}

#[test]
fn timeout_terminates_process_and_retains_diagnostics() -> Result<(), Box<dyn Error>> {
    let executable = std::env::current_exe()?;
    let child = Command::new(executable)
        .args([
            "--exact",
            "suite::runner::tests::timeout_child_fixture",
            "--nocapture",
        ])
        .env("FPAS_BENCH_TIMEOUT_FIXTURE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let started = Instant::now();
    let result = wait_for_output(Box::new(child), Duration::from_millis(100), "hung");
    let elapsed = started.elapsed();
    let error = match result {
        Ok(_) => return Err(io::Error::other("fixture should time out").into()),
        Err(error) => error,
    };

    assert_eq!(
        (
            error.contains("exceeded its 100 ms timeout"),
            error.contains("child started"),
            error.contains("child waiting"),
            elapsed < Duration::from_secs(5)
        ),
        (true, true, true, true)
    );
    Ok(())
}

#[test]
fn timeout_terminates_descendants_without_waiting_for_inherited_pipes() -> Result<(), Box<dyn Error>>
{
    let ready_path = std::env::temp_dir().join(format!(
        "fpas-bench-timeout-tree-ready-{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&ready_path);
    let executable = std::env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "suite::runner::tests::timeout_tree_parent_fixture",
            "--nocapture",
        ])
        .env("FPAS_BENCH_TIMEOUT_TREE_READY", &ready_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = spawn_benchmark(command)?;

    let ready_deadline = Instant::now() + Duration::from_secs(5);
    while !ready_path.is_file() && Instant::now() < ready_deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    if !ready_path.is_file() {
        return Err(io::Error::other("grandchild fixture did not start").into());
    }

    let started = Instant::now();
    let result = wait_for_output(child, Duration::from_millis(100), "process-tree");
    let elapsed = started.elapsed();
    let _ = fs::remove_file(&ready_path);
    let error = match result {
        Ok(_) => return Err(io::Error::other("fixture should time out").into()),
        Err(error) => error,
    };

    assert!(error.contains("grandchild started"), "{error}");
    assert!(
        elapsed < Duration::from_secs(1),
        "timeout waited {elapsed:?} for an inherited pipe"
    );
    Ok(())
}

#[test]
fn exited_parent_does_not_leave_descendant_pipe_open() -> Result<(), Box<dyn Error>> {
    let ready_path = std::env::temp_dir().join(format!(
        "fpas-bench-parent-exit-ready-{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&ready_path);
    let executable = std::env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .args([
            "--exact",
            "suite::runner::tests::timeout_tree_parent_fixture",
            "--nocapture",
        ])
        .env("FPAS_BENCH_TIMEOUT_TREE_READY", &ready_path)
        .env("FPAS_BENCH_TIMEOUT_TREE_PARENT_EXIT", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = spawn_benchmark(command)?;
    let started = Instant::now();
    let result = wait_for_output(child, Duration::from_secs(5), "parent-exit");
    let elapsed = started.elapsed();
    let _ = fs::remove_file(&ready_path);
    let output = result.map_err(io::Error::other)?;
    assert!(output.status.success());
    assert!(
        elapsed < Duration::from_secs(1),
        "inherited pipe stayed open for {elapsed:?}"
    );
    Ok(())
}

mod termination_failure;
