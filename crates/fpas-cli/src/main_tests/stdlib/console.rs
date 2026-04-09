use super::super::support;
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;

// ---------------------------------------------------------------------------
// Write / WriteLn basics
// ---------------------------------------------------------------------------

#[test]
fn writeln_string() {
    let source = support::program_with_console("  WriteLn('hello')\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
}

#[test]
fn writeln_integer() {
    let source = support::program_with_console("  WriteLn(42)\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn writeln_real() {
    let source = support::program_with_console("  WriteLn(3.14)\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3.14"), "got: {stdout}");
}

#[test]
fn writeln_boolean() {
    let source = support::program_with_console("  WriteLn(true);\n  WriteLn(false)\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\nfalse\n");
}

#[test]
fn writeln_no_args_emits_newline() {
    let source = support::program_with_console("  WriteLn\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "\n");
}

#[test]
fn writeln_multiple_args_variadic() {
    let source = support::program_with_console("  WriteLn('val=', 42, ' ok=', true)\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "val=42 ok=true\n");
}

#[test]
fn write_without_newline() {
    let source = support::program_with_console("  Write('a');\n  Write('b');\n  WriteLn\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ab\n");
}

// ---------------------------------------------------------------------------
// Fully qualified names
// ---------------------------------------------------------------------------

#[test]
fn fully_qualified_writeln() {
    let source = support::program_with_console("  Std.Console.WriteLn('qualified')\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "qualified\n");
}

#[test]
fn fully_qualified_write() {
    let source =
        support::program_with_console("  Std.Console.Write('a');\n  Std.Console.WriteLn\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "a\n");
}

// ---------------------------------------------------------------------------
// Negative: missing uses
// ---------------------------------------------------------------------------

#[test]
fn writeln_without_uses_is_error() {
    let source = support::program_without_uses("  WriteLn('hello')\n");
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", &source);
    assert_ne!(exit_code, 0);
    assert!(
        stdout.is_empty(),
        "expected no stdout on compile error, got: {stdout:?}"
    );
    assert!(
        stderr.contains(&format!("{SEMA_UNKNOWN_NAME}")),
        "expected unknown-name diagnostic in stderr, got: {stderr:?}"
    );
    assert!(
        stderr.contains("WriteLn"),
        "stderr should name the missing identifier: {stderr:?}"
    );
}
