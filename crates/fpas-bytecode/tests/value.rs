//! Runtime-value traversal regressions.

#![allow(
    clippy::expect_used,
    reason = "test-only child-process setup fails fast when the harness is unavailable"
)]

use std::process::{Command, Output};

use fpas_bytecode::Value;

const CHILD_CASE: &str = "FPAS_BYTECODE_DEEP_VALUE_CASE";
const VALUE_DEPTH: usize = 50_000;

fn nested_option(mut value: Value) -> Value {
    for _ in 0..VALUE_DEPTH {
        value = Value::OptionSome(Box::new(value));
    }
    value
}

fn run_child(case: &str) -> Output {
    Command::new(std::env::current_exe().expect("test executable must be available"))
        .args(["--exact", "deep_value_child", "--nocapture"])
        .env(CHILD_CASE, case)
        .output()
        .expect("deep-value child process must run")
}

#[test]
fn deep_value_equality_does_not_overflow_the_host_stack() {
    let output = run_child("equality");

    assert!(
        output.status.success(),
        "deep equality child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn deep_value_display_does_not_overflow_the_host_stack() {
    let output = run_child("display");

    assert!(
        output.status.success(),
        "deep display child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn deep_value_child() {
    let Ok(case) = std::env::var(CHILD_CASE) else {
        return;
    };

    match case.as_str() {
        "equality" => {
            let left = nested_option(Value::Integer(1));
            let right = nested_option(Value::Integer(1));
            let equal = left == right;
            std::mem::forget(left);
            std::mem::forget(right);
            assert!(equal);
        }
        "display" => {
            let value = nested_option(Value::Integer(1));
            let rendered_length = value.to_string().len();
            std::mem::forget(value);
            assert_eq!(rendered_length, VALUE_DEPTH * 6 + 1);
        }
        other => {
            eprintln!("unknown deep-value child case `{other}`");
            std::process::exit(2);
        }
    }
}
