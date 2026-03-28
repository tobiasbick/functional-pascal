use super::*;

#[test]
fn nested_if_inner_true() {
    let out = compile_and_run(
        "\
program T;
begin
  var A: boolean := true;
  var B: boolean := true;
  if A then
    if B then
      Std.Console.WriteLn('both true')
    else
      Std.Console.WriteLn('a true, b false')
end.",
    );
    assert_eq!(out.lines, vec!["both true"]);
}

#[test]
fn nested_if_inner_false() {
    let out = compile_and_run(
        "\
program T;
begin
  var A: boolean := true;
  var B: boolean := false;
  if A then
    if B then
      Std.Console.WriteLn('both true')
    else
      Std.Console.WriteLn('a true, b false')
end.",
    );
    assert_eq!(out.lines, vec!["a true, b false"]);
}

#[test]
fn nested_if_outer_false() {
    let out = compile_and_run(
        "\
program T;
begin
  var A: boolean := false;
  var B: boolean := true;
  if A then
    if B then
      Std.Console.WriteLn('both true')
    else
      Std.Console.WriteLn('a true, b false')
end.",
    );
    assert!(out.lines.is_empty());
}
