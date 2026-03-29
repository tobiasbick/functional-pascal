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

// ---------------------------------------------------------------------------
// Tan
// ---------------------------------------------------------------------------

#[test]
fn tan_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Tan(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// ArcSin / ArcCos / ArcTan
// ---------------------------------------------------------------------------

#[test]
fn arcsin_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcSin(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn arcsin_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(ArcSin(1.0) * 1000.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // Pi/2 * 1000 ≈ 1571
    assert_eq!(stdout, "1571\n");
}

#[test]
fn arcsin_out_of_range() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcSin(2.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn arccos_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcCos(1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn arccos_out_of_range() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcCos(-2.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn arctan_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcTan(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// ArcTan2
// ---------------------------------------------------------------------------

#[test]
fn arctan2_quarter_pi() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(ArcTan2(1.0, 1.0) * 1000.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // Pi/4 * 1000 ≈ 785
    assert_eq!(stdout, "785\n");
}

#[test]
fn arctan2_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(ArcTan2(0.0, 1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// Exp
// ---------------------------------------------------------------------------

#[test]
fn exp_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Exp(0.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn exp_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(Exp(1.0) * 1000.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // e * 1000 ≈ 2718
    assert_eq!(stdout, "2718\n");
}

// ---------------------------------------------------------------------------
// Log10 / Log2
// ---------------------------------------------------------------------------

#[test]
fn log10_hundred() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log10(100.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\n");
}

#[test]
fn log10_one() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log10(1.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn log10_non_positive_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log10(0.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn log2_eight() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log2(8.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn log2_non_positive_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Log2(-1.0))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// Trunc / Frac
// ---------------------------------------------------------------------------

#[test]
fn trunc_positive() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Trunc(3.9))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "3\n");
}

#[test]
fn trunc_negative() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Trunc(-3.7))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-3\n");
}

#[test]
fn trunc_whole_number() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Trunc(5.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "5\n");
}

#[test]
fn frac_positive() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(Frac(3.75) * 100.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "75\n");
}

#[test]
fn frac_negative() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Round(Frac(-3.75) * 100.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-75\n");
}

#[test]
fn frac_whole_number() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Frac(5.0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// Sign
// ---------------------------------------------------------------------------

#[test]
fn sign_positive_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(42))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn sign_negative_integer() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(-7))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

#[test]
fn sign_zero() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn sign_real() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(Sign(-3.14))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-1\n");
}

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

// ---------------------------------------------------------------------------
// Random / RandomInt / Randomize
// ---------------------------------------------------------------------------

#[test]
fn random_returns_value() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  var R: real := Random();
  WriteLn(R >= 0.0);
  WriteLn(R < 1.0)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "true\ntrue\n");
}

#[test]
fn random_int_returns_in_range() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  var N: integer := RandomInt(1, 1);
  WriteLn(N)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "1\n");
}

#[test]
fn random_int_reversed_bounds_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  WriteLn(RandomInt(2, 1))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn randomize_no_error() {
    let source = r#"program T;
uses Std.Console, Std.Math;
begin
  Randomize();
  WriteLn('ok')
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "ok\n");
}
