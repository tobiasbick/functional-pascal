use super::super::compile_and_run;
// ── Result.Map ──────────────────────────────────────────────────────────

#[test]
fn result_map_ok_transforms_value() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function DoubleToStr(V: integer): string;
begin
  return IntToStr(V * 2)
end;
begin
  var R: Result of integer, string := Ok(21);
  var M: Result of string, string := Map(R, DoubleToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn result_map_error_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result, Std.Conv;
function ToStr(V: integer): string;
begin
  return IntToStr(V)
end;
begin
  var R: Result of integer, string := Error('fail');
  var M: Result of string, string := Map(R, ToStr);
  case M of
    Ok(S): WriteLn(S);
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["fail"]);
}
