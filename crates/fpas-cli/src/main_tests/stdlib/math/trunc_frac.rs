use super::super::super::support;

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
