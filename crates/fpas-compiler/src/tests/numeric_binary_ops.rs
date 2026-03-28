use super::*;

#[test]
fn real_addition_prints() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  var x: real := 1.5;
  var y: real := 2.0;
  WriteLn(x + y)
end.",
    );
    assert_eq!(out.lines, vec!["3.5"]);
}

#[test]
fn real_literal_comparison_in_if() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  if 1.5 < 2.0 then
    WriteLn('yes')
  else
    WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}

#[test]
fn slash_on_integer_literals_is_real_division() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn(1 / 2)
end.",
    );
    assert_eq!(out.lines, vec!["0.5"]);
}

#[test]
fn integer_plus_real_promotes_to_real() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn(1 + 2.5)
end.",
    );
    assert_eq!(out.lines, vec!["3.5"]);
}

#[test]
fn mixed_numeric_comparison_uses_real_semantics() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  if 1 < 2.5 then
    WriteLn('ok')
end.",
    );
    assert_eq!(out.lines, vec!["ok"]);
}

#[test]
fn real_equality_comparison() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  if 2.0 = 2.0 then
    WriteLn('eq')
end.",
    );
    assert_eq!(out.lines, vec!["eq"]);
}
