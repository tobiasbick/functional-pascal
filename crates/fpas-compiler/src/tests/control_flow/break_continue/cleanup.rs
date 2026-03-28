use super::*;

#[test]
fn break_pops_multiple_locals() {
    let out = compile_and_run(
        "\
program BrkPopLocals;
begin
  for I: integer := 1 to 10 do
  begin
    var A: integer := I;
    var B: string := 'x';
    var C: integer := I * 10;
    if I = 3 then
      break;
    Std.Console.WriteLn(C)
  end;
  Std.Console.WriteLn('after')
end.",
    );
    assert_eq!(out.lines, vec!["10", "20", "after"]);
}

#[test]
fn continue_pops_multiple_locals() {
    let out = compile_and_run(
        "\
program ContPopLocals;
begin
  mutable var Sum: integer := 0;
  for I: integer := 1 to 5 do
  begin
    var A: integer := I;
    var B: integer := I * 2;
    if I mod 2 = 0 then
      continue;
    Sum := Sum + A + B
  end;
  Std.Console.WriteLn(Sum)
end.",
    );
    assert_eq!(out.lines, vec!["27"]);
}
