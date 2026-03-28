use super::super::support;

// ---------------------------------------------------------------------------
// Pi constant
// ---------------------------------------------------------------------------

#[test]
fn pi_constant() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pi)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3.14"), "got: {stdout}");
}

#[test]
fn pi_fully_qualified() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Std.Math.Pi)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3.14"), "got: {stdout}");
}

// ---------------------------------------------------------------------------
// Sqrt
// ---------------------------------------------------------------------------

#[test]
fn sqrt_normal() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(16.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "4\n");
}

#[test]
fn sqrt_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn sqrt_negative_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sqrt(-1.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Pow
// ---------------------------------------------------------------------------

#[test]
fn pow_normal() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pow(2.0, 3.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "8\n");
}

#[test]
fn pow_zero_exponent() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Pow(5.0, 0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

// ---------------------------------------------------------------------------
// Floor / Ceil / Round
// ---------------------------------------------------------------------------

#[test]
fn floor_positive() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Floor(2.9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn floor_negative() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Floor(-2.1))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-3\n");
}

#[test]
fn ceil_positive() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Ceil(2.1))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn ceil_negative() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Ceil(-2.9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-2\n");
}

#[test]
fn round_normal() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(2.6))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

// ---------------------------------------------------------------------------
// Sin / Cos
// ---------------------------------------------------------------------------

#[test]
fn sin_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sin(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn cos_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Cos(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

// ---------------------------------------------------------------------------
// Log
// ---------------------------------------------------------------------------

#[test]
fn log_e() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(Log(2.718281828459045)))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn log_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log(1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn log_zero_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log(0.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn log_negative_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log(-1.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Abs (integer and real)
// ---------------------------------------------------------------------------

#[test]
fn abs_negative_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(-7))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "7\n");
}

#[test]
fn abs_positive_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn abs_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn abs_negative_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Abs(-1.5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("1.5"), "got: {stdout}");
}

// ---------------------------------------------------------------------------
// Min / Max
// ---------------------------------------------------------------------------

#[test]
fn min_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(3, 9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn max_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(3, 9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "9\n");
}

#[test]
fn min_equal_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(5, 5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn max_equal_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(5, 5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn min_negative_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Min(-10, -3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-10\n");
}

#[test]
fn max_negative_values() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Max(-10, -3))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-3\n");
}
