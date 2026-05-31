use super::super::super::support;

#[test]
fn json_parse_null_maps_to_jsonvalue_null() {
    let source = r#"program T;
uses Std.Console, Std.Json;
begin
  var R: Result of JsonValue, string := Parse('null');
  case R of
    Ok(V):
      case V of
        JsonValue.Null: WriteLn('null');
        else WriteLn('not-null')
      end;
    Error(E): WriteLn('error: ' + E)
  end
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "null\n");
}

#[test]
fn json_parse_nested_object_and_array() {
    let source = r#"program T;
uses Std.Console, Std.Json, Std.Dict, Std.Option, Std.Conv, Std.Array;
begin
  var R: Result of JsonValue, string := Parse('{"name":"Ada","scores":[1,2],"active":true}');
  case R of
    Ok(V):
      case V of
        JsonValue.Object(Fields): begin
          var NameValue: Option of JsonValue := Std.Dict.Get(Fields, 'name');
          var ScoresValue: Option of JsonValue := Std.Dict.Get(Fields, 'scores');
          var ActiveValue: Option of JsonValue := Std.Dict.Get(Fields, 'active');
          case Unwrap(NameValue) of
            JsonValue.String(Name): WriteLn(Name);
            else WriteLn('bad-name')
          end;
          case Unwrap(ScoresValue) of
            JsonValue.Array(Scores): WriteLn(IntToStr(Std.Array.Length(Scores)));
            else WriteLn('bad-scores')
          end;
          case Unwrap(ActiveValue) of
            JsonValue.Bool(Active): WriteLn(Active);
            else WriteLn('bad-active')
          end
        end;
        else WriteLn('not-object')
      end;
    Error(E): WriteLn('error: ' + E)
  end
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "Ada\n2\ntrue\n");
}

#[test]
fn json_parse_invalid_json_returns_error() {
    let source = r#"program T;
uses Std.Console, Std.Json;
begin
  var R: Result of JsonValue, string := Parse('{bad json');
  case R of
    Ok(V): WriteLn(Stringify(V));
    Error(E): WriteLn('error')
  end
end.
"#;
    let (exit_code, stdout, stderr) = support::run_source_and_capture_output("t.fpas", source);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "error\n");
}
