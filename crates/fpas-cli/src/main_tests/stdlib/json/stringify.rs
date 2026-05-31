use super::super::super::support;

#[test]
fn json_stringify_constructed_array() {
    let source = r#"program T;
uses Std.Console, Std.Json;
begin
  var V: JsonValue := JsonValue.Array([
    JsonValue.Bool(true),
    JsonValue.Null,
    JsonValue.String('hi'),
    JsonValue.Number(1.5)
  ]);
  WriteLn(Stringify(V))
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "[true,null,\"hi\",1.5]\n");
}

#[test]
fn json_stringify_parse_roundtrip() {
    let source = r#"program T;
uses Std.Console, Std.Json;
begin
  var R: Result of JsonValue, string := Parse('[false,null,"x"]');
  case R of
    Ok(V): WriteLn(Stringify(V));
    Error(E): WriteLn('error: ' + E)
  end
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "[false,null,\"x\"]\n");
}
