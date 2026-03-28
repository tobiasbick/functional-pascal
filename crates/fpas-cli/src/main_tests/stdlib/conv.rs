use super::super::support;

// ---------------------------------------------------------------------------
// IntToStr
// ---------------------------------------------------------------------------

#[test]
fn int_to_str_positive() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(42))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "42\n");
}

#[test]
fn int_to_str_negative() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(-7))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-7\n");
}

#[test]
fn int_to_str_zero() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(0))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

// ---------------------------------------------------------------------------
// StrToInt
// ---------------------------------------------------------------------------

#[test]
fn str_to_int_valid() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToInt('123'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "123\n");
}

#[test]
fn str_to_int_with_whitespace() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToInt('  -7  '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "-7\n");
}

#[test]
fn str_to_int_invalid_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToInt('abc'))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn str_to_int_empty_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToInt(''))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// IntToReal
// ---------------------------------------------------------------------------

#[test]
fn int_to_real() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  var X: real := IntToReal(3);
  WriteLn(X)
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.starts_with("3"), "got: {stdout}");
}

// ---------------------------------------------------------------------------
// RealToStr
// ---------------------------------------------------------------------------

#[test]
fn real_to_str() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(RealToStr(1.5))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("1.5"), "got: {stdout}");
}

// ---------------------------------------------------------------------------
// StrToReal
// ---------------------------------------------------------------------------

#[test]
fn str_to_real_valid() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToReal('2.25'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert!(stdout.contains("2.25"), "got: {stdout}");
}

#[test]
fn str_to_real_invalid_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToReal('abc'))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn str_to_real_empty_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(StrToReal(''))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

// ---------------------------------------------------------------------------
// CharToStr
// ---------------------------------------------------------------------------

#[test]
fn char_to_str() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(CharToStr('Z'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Z\n");
}

// ---------------------------------------------------------------------------
// Fully qualified names
// ---------------------------------------------------------------------------

#[test]
fn fully_qualified_int_to_str() {
    let source = r#"program T;
uses Std.Console, Std.Conv;
begin
  WriteLn(Std.Conv.IntToStr(99))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "99\n");
}
