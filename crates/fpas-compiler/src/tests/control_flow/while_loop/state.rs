use super::*;

#[test]
fn while_accumulator() {
    let out = compile_and_run(
        "\
program WhileAccum;
begin
  mutable var Sum: integer := 0;
  mutable var I: integer := 1;
  while I <= 5 do
  begin
    Sum := Sum + I;
    I := I + 1
  end;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}
