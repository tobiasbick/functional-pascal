use super::super::{compile_and_run, compile_err};
// ── Try with Option — FirstPositive example from docs ───────────────────

#[test]
fn try_option_first_positive_found() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
function FirstPositive(Items: array of integer): Option of integer;
begin
  var Idx: integer := try FindIndex(Items, 1);
  return Some(Items[Idx])
end;
begin
  case FirstPositive([0, 1, 2]) of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}

#[test]
fn try_option_first_positive_not_found() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
function FirstPositive(Items: array of integer): Option of integer;
begin
  var Idx: integer := try FindIndex(Items, 1);
  return Some(Items[Idx])
end;
begin
  case FirstPositive([0, 2, 3]) of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}
