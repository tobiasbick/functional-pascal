use super::*;

#[test]
fn for_to_basic() {
    let out = compile_and_run(
        "\
program ForTo;
begin
  for I: integer := 1 to 5 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3", "4", "5"]);
}

#[test]
fn for_to_with_block() {
    let out = compile_and_run(
        "\
program ForToBlock;
begin
  for I: integer := 1 to 3 do
  begin
    var Doubled: integer := I * 2;
    Std.Console.WriteLn(Doubled)
  end
end.",
    );
    assert_eq!(out.lines, vec!["2", "4", "6"]);
}

#[test]
fn for_downto_basic() {
    let out = compile_and_run(
        "\
program ForDownto;
begin
  for I: integer := 5 downto 1 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["5", "4", "3", "2", "1"]);
}

#[test]
fn for_downto_with_block() {
    let out = compile_and_run(
        "\
program ForDowntoBlock;
begin
  for I: integer := 3 downto 1 do
  begin
    var Sq: integer := I * I;
    Std.Console.WriteLn(Sq)
  end
end.",
    );
    assert_eq!(out.lines, vec!["9", "4", "1"]);
}

#[test]
fn for_to_single_iteration() {
    let out = compile_and_run(
        "\
program ForToSingle;
begin
  for I: integer := 7 to 7 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}

#[test]
fn for_downto_single_iteration() {
    let out = compile_and_run(
        "\
program ForDowntoSingle;
begin
  for I: integer := 7 downto 7 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["7"]);
}
