use super::*;

#[test]
fn for_to_zero_iterations_when_start_greater_than_end() {
    let out = compile_and_run(
        "\
program ForToZero;
begin
  mutable var Count: integer := 0;
  for I: integer := 5 to 1 do
    Count := Count + 1;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_downto_zero_iterations_when_start_less_than_end() {
    let out = compile_and_run(
        "\
program ForDowntoZero;
begin
  mutable var Count: integer := 0;
  for I: integer := 1 downto 5 do
    Count := Count + 1;
  Std.Console.WriteLn(Count)
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn for_to_negative_range() {
    let out = compile_and_run(
        "\
program ForNeg;
begin
  for I: integer := -3 to 0 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["-3", "-2", "-1", "0"]);
}

#[test]
fn for_downto_negative_range() {
    let out = compile_and_run(
        "\
program ForDownNeg;
begin
  for I: integer := 0 downto -3 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["0", "-1", "-2", "-3"]);
}

#[test]
fn for_to_entirely_negative() {
    let out = compile_and_run(
        "\
program ForNegNeg;
begin
  for I: integer := -5 to -2 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["-5", "-4", "-3", "-2"]);
}

#[test]
fn for_downto_entirely_negative() {
    let out = compile_and_run(
        "\
program ForDownNegNeg;
begin
  for I: integer := -2 downto -5 do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["-2", "-3", "-4", "-5"]);
}

#[test]
fn for_to_with_expression_bounds() {
    let out = compile_and_run(
        "\
program ForExprBounds;
begin
  var Start: integer := 2;
  var Finish: integer := 4;
  for I: integer := Start to Finish do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["2", "3", "4"]);
}

#[test]
fn for_downto_with_expression_bounds() {
    let out = compile_and_run(
        "\
program ForDownExprBounds;
begin
  var High: integer := 4;
  var Low: integer := 2;
  for I: integer := High downto Low do
    Std.Console.WriteLn(I)
end.",
    );
    assert_eq!(out.lines, vec!["4", "3", "2"]);
}
