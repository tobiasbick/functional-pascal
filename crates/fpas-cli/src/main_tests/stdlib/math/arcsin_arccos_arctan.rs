use super::super::super::support;

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
