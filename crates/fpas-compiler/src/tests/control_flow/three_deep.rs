use super::*;
// three levels deep.
#[test]
fn for_in_repeat_in_while() {
    let out = compile_and_run(
        "\
program ThreeDeep;
begin
  mutable var A: integer := 0;
  while A < 2 do
  begin
    mutable var B: integer := 0;
    repeat
      for C: integer := 1 to 2 do
        Std.Console.WriteLn(A * 100 + B * 10 + C);
      B := B + 1
    until B = 2;
    A := A + 1
  end
end.",
    );
    assert_eq!(
        out.lines,
        vec!["1", "2", "11", "12", "101", "102", "111", "112"]
    );
}
#[test]
fn repeat_in_while_in_for() {
    let out = compile_and_run(
        "\
program ThreeDeepB;
begin
  for A: integer := 1 to 2 do
  begin
    mutable var B: integer := 0;
    while B < 2 do
    begin
      mutable var C: integer := 0;
      repeat
        Std.Console.WriteLn(A * 100 + B * 10 + C);
        C := C + 1
      until C = 2;
      B := B + 1
    end
  end
end.",
    );
    assert_eq!(
        out.lines,
        vec!["100", "101", "110", "111", "200", "201", "210", "211"]
    );
}
#[test]
fn while_in_for_in_repeat() {
    let out = compile_and_run(
        "\
program ThreeDeepC;
begin
  mutable var A: integer := 0;
  repeat
    for B: integer := 1 to 2 do
    begin
      mutable var C: integer := 0;
      while C < 2 do
      begin
        Std.Console.WriteLn(A * 100 + B * 10 + C);
        C := C + 1
      end
    end;
    A := A + 1
  until A = 2
end.",
    );
    assert_eq!(
        out.lines,
        vec!["10", "11", "20", "21", "110", "111", "120", "121"]
    );
}
