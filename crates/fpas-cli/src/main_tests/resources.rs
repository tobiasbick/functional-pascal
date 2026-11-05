//! Public options must reach ordinary runs and isolated test/debug workers.

use super::support::run_cli_args_and_capture_output;
use super::*;

const GROW: &str = "program Grow; begin var Text: string := 'x'; while true do Text := Text + Text; end while; end.";

#[test]
fn resource_options_are_shared_by_run_test_and_debug() {
    let cwd = create_temp_dir("execution-options");
    write_text(&cwd.join("grow_test.fpas"), GROW);
    for command in ["run", "test", "debug"] {
        let args = [
            command,
            "grow_test.fpas",
            "--heap-limit",
            "16384",
            "--instruction-limit",
            "1000",
            "--output-limit",
            "128",
            "--timeout",
            "2",
        ]
        .map(str::to_owned);
        let parsed = resolve_cli_config(&args, &cwd).unwrap();
        match parsed {
            ResolvedCli::Run(config) => {
                assert_eq!(config.execution_limits.max_heap_bytes, Some(16384))
            }
            ResolvedCli::Test(config) => {
                assert_eq!(config.execution_limits.max_instructions, Some(1000))
            }
            ResolvedCli::Debug(config) => assert_eq!(config.heap_limit, Some(16384)),
            other => panic!("unexpected command {other:?}"),
        }
        for invalid in ["0", "-1", "wrong", "18446744073709551616"] {
            let args = [command, "grow_test.fpas", "--heap-limit", invalid].map(str::to_owned);
            assert!(resolve_cli_config(&args, &cwd).is_err());
        }
        let args = [
            command,
            "grow_test.fpas",
            "--heap-limit",
            "10",
            "--heap-limit",
            "20",
        ]
        .map(str::to_owned);
        assert!(
            resolve_cli_config(&args, &cwd)
                .unwrap_err()
                .contains("Duplicate")
        );
    }
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn run_reports_a_regular_runtime_diagnostic_for_heap_and_instruction_limits() {
    let cwd = create_temp_dir("execution-limits-run");
    write_text(&cwd.join("grow.fpas"), GROW);
    write_text(
        &cwd.join("loop.fpas"),
        "program Loop; begin while true do null; end while; end.",
    );
    for (file, option, limit) in [
        ("grow.fpas", "--heap-limit", "16384"),
        ("loop.fpas", "--instruction-limit", "1000"),
    ] {
        let args = ["run", file, option, limit].map(str::to_owned);
        let (exit, output, diagnostics) = run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 2, "{diagnostics}");
        assert!(output.is_empty());
        assert!(diagnostics.contains("FP5029"), "{diagnostics}");
    }
    fs::remove_dir_all(cwd).unwrap();
}

#[test]
fn isolated_tests_receive_the_heap_limit() {
    let cwd = create_temp_dir("execution-limits-test");
    write_text(&cwd.join("grow_test.fpas"), GROW);
    let args = [
        "test",
        "grow_test.fpas",
        "--heap-limit",
        "16384",
        "--timeout",
        "5",
    ]
    .map(str::to_owned);
    let (exit, _, diagnostics) = run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(cwd).unwrap();
    assert_ne!(exit, 0);
    assert!(diagnostics.contains("FP5029"), "{diagnostics}");
    assert!(!diagnostics.contains("exceeded 5 second"), "{diagnostics}");
}

#[test]
fn console_output_is_limited_in_native_console_workers() {
    let cwd = create_temp_dir("execution-limits-output");
    write_text(
        &cwd.join("print.fpas"),
        "program Print; uses Std.Console as Console; begin Console.WriteText('abc'); Console.WriteLn('z'); end.",
    );
    let args = ["run", "print.fpas", "--output-limit", "3"].map(str::to_owned);
    let (exit, output, diagnostics) = run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(cwd).unwrap();
    assert_eq!(exit, 2, "{diagnostics}");
    assert_eq!(output, "abc");
    assert!(diagnostics.contains("FP5029"), "{diagnostics}");
}

#[test]
fn isolated_debug_execution_receives_the_heap_limit() {
    let cwd = create_temp_dir("execution-limits-debug");
    write_text(&cwd.join("grow.fpas"), GROW);
    let args = [
        "debug",
        "grow.fpas",
        "--heap-limit",
        "16384",
        "--timeout",
        "5",
    ]
    .map(str::to_owned);
    let (exit, output, diagnostics) = run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(cwd).unwrap();
    assert_ne!(exit, 0, "{output} {diagnostics}");
    assert!(output.contains("heap_limit"), "{output} {diagnostics}");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["outcome"], "limit");
}
