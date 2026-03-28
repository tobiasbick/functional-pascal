use super::*;

#[test]
fn for_to_accumulator() {
    let out = compile_and_run(
        "\
program ForAccum;
begin
  mutable var Sum: integer := 0;
  for I: integer := 1 to 10 do
    Sum := Sum + I;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["55"]);
}

#[test]
fn for_downto_accumulator() {
    let out = compile_and_run(
        "\
program ForDownAccum;
begin
  mutable var Product: integer := 1;
  for I: integer := 5 downto 1 do
    Product := Product * I;
  Std.Console.WriteLn(Product)
end.",
    );
    assert_eq!(out.lines, vec!["120"]);
}
