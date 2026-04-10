use super::super::super::support;

// ---------------------------------------------------------------------------
// FlatMap
// ---------------------------------------------------------------------------

#[test]
fn flat_map_normal() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function Expand(X: integer): array of integer;
begin
  return [X, X * 10]
end;
begin
  var R: array of integer := FlatMap([1, 2, 3], Expand);
  WriteLn(Length(R));
  WriteLn(R[0]);
  WriteLn(R[1]);
  WriteLn(R[4]);
  WriteLn(R[5])
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "6\n1\n10\n3\n30\n");
}

#[test]
fn flat_map_empty_results() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function EmptyArray(X: integer): array of integer;
begin
  return []
end;
begin
  var R: array of integer := FlatMap([1, 2, 3], EmptyArray);
  WriteLn(Length(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn flat_map_empty_array() {
    let source = r#"program T;
uses Std.Console, Std.Array;
function Wrap(X: integer): array of integer;
begin
  return [X]
end;
begin
  var R: array of integer := FlatMap([], Wrap);
  WriteLn(Length(R))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "0\n");
}

#[test]
fn flat_map_rejects_scalar_mapper_result() {
    let source = r#"program T;
uses Std.Array;
function Identity(V: integer): integer;
begin
  return V
end;
begin
  var X: integer := FlatMap([1, 2], Identity)
end.
"#;
    let (exit_code, _stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
    assert!(
        stderr.contains("mapper must return an array"),
        "stderr: {stderr}"
    );
}
