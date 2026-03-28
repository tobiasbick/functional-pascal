use super::*;

#[test]
fn multiple_sequential_ifs() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 5;
  if X > 0 then
    Std.Console.WriteLn('positive');
  if X > 3 then
    Std.Console.WriteLn('above three');
  if X > 10 then
    Std.Console.WriteLn('above ten')
end.",
    );
    assert_eq!(out.lines, vec!["positive", "above three"]);
}
