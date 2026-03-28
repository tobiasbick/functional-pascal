use super::*;

#[test]
fn nested_for_in() {
    let out = compile_and_run(
        "\
program NestedForIn;
begin
  var Outer: array of integer := [1, 2];
  var Inner: array of integer := [10, 20];
  for A: integer in Outer do
    for B: integer in Inner do
      Std.Console.WriteLn(A * 100 + B)
end.",
    );
    assert_eq!(out.lines, vec!["110", "120", "210", "220"]);
}

#[test]
fn nested_for_in_break_inner_only() {
    let out = compile_and_run(
        "\
program NestedBreakInner;
begin
  var Outer: array of integer := [1, 2, 3];
  var Inner: array of integer := [10, 20, 30];
  for A: integer in Outer do
  begin
    for B: integer in Inner do
    begin
      if B = 20 then break;
      Std.Console.WriteLn(A * 100 + B)
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["110", "210", "310"]);
}

#[test]
fn nested_for_in_continue_inner_only() {
    let out = compile_and_run(
        "\
program NestedContInner;
begin
  var Outer: array of integer := [1, 2];
  var Inner: array of integer := [10, 20, 30];
  for A: integer in Outer do
    for B: integer in Inner do
    begin
      if B = 20 then continue;
      Std.Console.WriteLn(A * 100 + B)
    end
end.",
    );
    assert_eq!(out.lines, vec!["110", "130", "210", "230"]);
}

#[test]
fn for_in_inside_classic_for() {
    let out = compile_and_run(
        "\
program ForInInsideFor;
begin
  var Arr: array of string := ['a', 'b'];
  for I: integer := 1 to 2 do
    for S: string in Arr do
      Std.Console.WriteLn(S)
end.",
    );
    assert_eq!(out.lines, vec!["a", "b", "a", "b"]);
}

#[test]
fn classic_for_inside_for_in() {
    let out = compile_and_run(
        "\
program ForInsideForIn;
begin
  var Arr: array of integer := [10, 20];
  for Base: integer in Arr do
    for I: integer := 1 to 3 do
      Std.Console.WriteLn(Base + I)
end.",
    );
    assert_eq!(out.lines, vec!["11", "12", "13", "21", "22", "23"]);
}
