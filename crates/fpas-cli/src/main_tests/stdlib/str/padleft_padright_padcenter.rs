use super::super::super::support;

// ---------------------------------------------------------------------------
// PadLeft / PadRight / PadCenter
// ---------------------------------------------------------------------------

#[test]
fn pad_left_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadLeft('42', 5, '0'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "00042\n");
}

#[test]
fn pad_left_already_wide() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadLeft('Hello', 3, ' '))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn pad_right_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadRight('Hi', 6, '.'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hi....\n");
}

#[test]
fn pad_right_already_wide() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadRight('Hello', 2, '.'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn pad_center_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('Hi', 6, '-'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "--Hi--\n");
}

#[test]
fn pad_center_odd_spacing() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('Hi', 7, '-'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    // 7 - 2 = 5 remaining, left gets 2, right gets 3
    assert_eq!(stdout, "--Hi---\n");
}

#[test]
fn pad_center_already_wide() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('Hello', 3, '-'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Hello\n");
}

#[test]
fn pad_left_negative_width_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadLeft('a', -1, '.'))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("PadLeft width must be >= 0"),
        "stderr: {stderr}"
    );
}

#[test]
fn pad_right_negative_width_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadRight('a', -1, '.'))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("PadRight width must be >= 0"),
        "stderr: {stderr}"
    );
}

#[test]
fn pad_center_negative_width_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(PadCenter('a', -1, '.'))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("PadCenter width must be >= 0"),
        "stderr: {stderr}"
    );
}
