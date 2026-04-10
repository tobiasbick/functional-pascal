use super::super::super::support;

// ---------------------------------------------------------------------------
// Clamp
// ---------------------------------------------------------------------------

#[test]
fn clamp_in_range() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(50, 0, 100))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "50\n");
}

#[test]
fn clamp_below() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(-5, 0, 100))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn clamp_above() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(150, 0, 100))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "100\n");
}

#[test]
fn clamp_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(1.5, 0.0, 1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn clamp_mixed_types_is_compile_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(5, 0, 10.0))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(stderr.contains("same numeric kind"), "stderr: {stderr}");
}

#[test]
fn min_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(3.5, 1.2));
  WriteLn(Min(-0.5, 0.5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1.2\n-0.5\n");
}

#[test]
fn max_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(3.5, 1.2));
  WriteLn(Max(-0.5, 0.5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3.5\n0.5\n");
}

#[test]
fn min_mixed_types_is_compile_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(3, 1.5))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(stderr.contains("same numeric kind"), "stderr: {stderr}");
}

#[test]
fn max_mixed_types_is_compile_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(3, 1.5))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(stderr.contains("same numeric kind"), "stderr: {stderr}");
}

#[test]
fn clamp_at_boundary() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(0, 0, 100));
  WriteLn(Clamp(100, 0, 100))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n100\n");
}

#[test]
fn clamp_reversed_bounds_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Clamp(1, 10, 0))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("Clamp lower bound 10 must be <= upper bound 0"),
        "stderr: {stderr}"
    );
}
