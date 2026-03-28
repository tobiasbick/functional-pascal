use super::*;

#[test]
fn for_in_string_array() {
    let out = compile_and_run(
        "\
program ForInStrings;
begin
  var Names: array of string := ['Alice', 'Bob', 'Charlie'];
  for Name: string in Names do
    Std.Console.WriteLn(Name)
end.",
    );
    assert_eq!(out.lines, vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn for_in_boolean_array() {
    let out = compile_and_run(
        "\
program ForInBool;
begin
  var Flags: array of boolean := [true, false, true];
  for F: boolean in Flags do
    Std.Console.WriteLn(F)
end.",
    );
    assert_eq!(out.lines, vec!["true", "false", "true"]);
}

#[test]
fn for_in_real_array() {
    let out = compile_and_run(
        "\
program ForInReal;
begin
  var Vals: array of real := [1.5, 2.5, 3.5];
  for V: real in Vals do
    Std.Console.WriteLn(V)
end.",
    );
    assert_eq!(out.lines, vec!["1.5", "2.5", "3.5"]);
}
