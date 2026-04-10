use super::super::super::support;

// ---------------------------------------------------------------------------
// Edge: array index out of bounds
// ---------------------------------------------------------------------------

#[test]
fn array_index_out_of_bounds_is_runtime_error() {
    let source = r#"program T;
uses Std.Console, Std.Array;
begin
  var A: array of integer := [1, 2];
  WriteLn(A[5])
end.
"#;
    let (exit_code, _stdout, _stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert_ne!(exit_code, 0);
}
