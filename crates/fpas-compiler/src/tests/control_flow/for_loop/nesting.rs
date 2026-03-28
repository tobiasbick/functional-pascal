use super::*;

#[test]
fn nested_for_to_to() {
    let out = compile_and_run(
        "\
program NestedForTo;
begin
  for I: integer := 1 to 3 do
    for J: integer := 1 to 2 do
      Std.Console.WriteLn(I * 10 + J)
end.",
    );
    assert_eq!(out.lines, vec!["11", "12", "21", "22", "31", "32"]);
}

#[test]
fn nested_for_downto_outer_to_inner() {
    let out = compile_and_run(
        "\
program NestedForDownToUp;
begin
  for I: integer := 3 downto 1 do
    for J: integer := 1 to 2 do
      Std.Console.WriteLn(I * 10 + J)
end.",
    );
    assert_eq!(out.lines, vec!["31", "32", "21", "22", "11", "12"]);
}

#[test]
fn nested_for_inner_break_only() {
    let out = compile_and_run(
        "\
program NestedForInnerBreak;
begin
  for I: integer := 1 to 3 do
    for J: integer := 1 to 10 do
    begin
      if J > 2 then break;
      Std.Console.WriteLn(I * 10 + J)
    end
end.",
    );
    assert_eq!(out.lines, vec!["11", "12", "21", "22", "31", "32"]);
}
