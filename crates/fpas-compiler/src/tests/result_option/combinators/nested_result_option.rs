use super::super::compile_and_run;
// ── Nested Result/Option ────────────────────────────────────────────────

#[test]
fn result_map_nested_option_inner() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Option, Std.Conv;
function WrapInSome(V: integer): Option of integer;
begin
  return Some(V * 2)
end;
begin
  var R: Result of integer, string := Ok(5);
  var M: Result of Option of integer, string := Std.Result.Map(R, WrapInSome);
  case M of
    Ok(Inner):
      case Inner of
        Some(V): WriteLn(IntToStr(V));
        None: WriteLn('inner-none')
      end;
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["10"]);
}

#[test]
fn option_map_wraps_in_result() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Option, Std.Conv;
function WrapInOk(V: integer): Result of integer, string;
begin
  return Ok(V * 4)
end;
begin
  var O: Option of integer := Some(3);
  var M: Option of Result of integer, string := Std.Option.Map(O, WrapInOk);
  case M of
    Some(Inner):
      case Inner of
        Ok(V): WriteLn(IntToStr(V));
        Error(E): WriteLn(E)
      end;
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["12"]);
}
