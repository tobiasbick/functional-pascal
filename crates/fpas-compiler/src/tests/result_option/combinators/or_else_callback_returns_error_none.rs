use super::super::compile_and_run;
// ── OrElse callback returns Error/None ──────────────────────────────────

#[test]
fn result_or_else_error_to_error() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Result;
function ReplaceError(E: string): Result of integer, string;
begin
  return Error('replaced: ' + E)
end;
begin
  var R: Result of integer, string := Error('first');
  var M: Result of integer, string := OrElse(R, ReplaceError);
  case M of
    Ok(V): WriteLn('ok');
    Error(E): WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["replaced: first"]);
}

#[test]
fn option_or_else_none_returns_none() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option;
function AlwaysNone(): Option of integer;
begin
  return None
end;
begin
  var O: Option of integer := None;
  var M: Option of integer := OrElse(O, AlwaysNone);
  case M of
    Some(V): WriteLn('some');
    None: WriteLn('still-none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["still-none"]);
}
