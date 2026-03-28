use super::*;

#[test]
fn code_after_for_in_executes() {
    let out = compile_and_run(
        "\
program ForInAfter;
begin
  var Arr: array of integer := [1, 2];
  for X: integer in Arr do
    Std.Console.WriteLn(X);
  Std.Console.WriteLn('done')
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "done"]);
}

#[test]
fn code_after_empty_for_in_executes() {
    let out = compile_and_run(
        "\
program ForInEmptyAfter;
begin
  var Arr: array of integer := [];
  for X: integer in Arr do
    Std.Console.WriteLn(X);
  Std.Console.WriteLn('still runs')
end.",
    );
    assert_eq!(out.lines, vec!["still runs"]);
}
