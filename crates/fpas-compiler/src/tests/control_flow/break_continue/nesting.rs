use super::*;

#[test]
fn nested_break_outer_via_flag() {
    let out = compile_and_run(
        "\
program NestedBrkOuter;
begin
  mutable var Done: boolean := false;
  mutable var I: integer := 0;
  while not Done do
  begin
    I := I + 1;
    for J: integer := 1 to 5 do
    begin
      if (I = 2) and (J = 3) then
      begin
        Done := true;
        break
      end;
      Std.Console.WriteLn(I * 10 + J)
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["11", "12", "13", "14", "15", "21", "22"]);
}

#[test]
fn nested_continue_outer_skip_inner() {
    let out = compile_and_run(
        "\
program ContOuter;
begin
  mutable var I: integer := 0;
  while I < 4 do
  begin
    I := I + 1;
    if I = 2 then
      continue;
    for J: integer := 1 to 2 do
      Std.Console.WriteLn(I * 10 + J)
  end
end.",
    );
    assert_eq!(out.lines, vec!["11", "12", "31", "32", "41", "42"]);
}
