use super::super::compile_and_run;
// ── Option.OrElse ───────────────────────────────────────────────────────

#[test]
fn option_or_else_some_passes_through() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function FallbackZero(): Option of integer;
begin
  return Some(0)
end;
begin
  var O: Option of integer := Some(42);
  var M: Option of integer := OrElse(O, FallbackZero);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn option_or_else_none_provides_fallback() {
    let out = compile_and_run(
        "program T;
uses Std.Console, Std.Option, Std.Conv;
function FallbackNinetyNine(): Option of integer;
begin
  return Some(99)
end;
begin
  var O: Option of integer := None;
  var M: Option of integer := OrElse(O, FallbackNinetyNine);
  case M of
    Some(V): WriteLn(IntToStr(V));
    None: WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}
