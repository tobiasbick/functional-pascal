use super::super::super::support;

// ---------------------------------------------------------------------------
// Ord / Chr
// ---------------------------------------------------------------------------

#[test]
fn ord_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Ord('A'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "65\n");
}

#[test]
fn ord_zero_char() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Ord('0'))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "48\n");
}

#[test]
fn chr_normal() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(65))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "A\n");
}

#[test]
fn chr_invalid_codepoint() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(-1))
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}

#[test]
fn chr_rejects_oversized_codepoint() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(4294967296))
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("Chr: 4294967296 is not a valid Unicode code point"),
        "stderr: {stderr}"
    );
}

#[test]
fn ord_chr_roundtrip() {
    let source = r#"program T;
uses Std.Console, Std.Str;
begin
  WriteLn(Chr(Ord('Z')))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Z\n");
}
