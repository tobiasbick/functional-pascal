use super::*;

#[test]
fn if_string_equal() {
    let out = compile_and_run(
        "\
program T;
begin
  var Name: string := 'Alice';
  if Name = 'Alice' then
    Std.Console.WriteLn('found')
  else
    Std.Console.WriteLn('not found')
end.",
    );
    assert_eq!(out.lines, vec!["found"]);
}

#[test]
fn if_string_not_equal() {
    let out = compile_and_run(
        "\
program T;
begin
  var Name: string := 'Bob';
  if Name = 'Alice' then
    Std.Console.WriteLn('found')
  else
    Std.Console.WriteLn('not found')
end.",
    );
    assert_eq!(out.lines, vec!["not found"]);
}
