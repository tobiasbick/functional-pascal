use super::*;

#[test]
fn if_true_executes_then() {
    let out = compile_and_run(
        "\
program T;
begin
  if true then
    Std.Console.WriteLn('then')
end.",
    );
    assert_eq!(out.lines, vec!["then"]);
}

#[test]
fn if_false_skips_then() {
    let out = compile_and_run(
        "\
program T;
begin
  if false then
    Std.Console.WriteLn('should not appear')
end.",
    );
    assert!(out.lines.is_empty());
}

#[test]
fn if_true_takes_then_not_else() {
    let out = compile_and_run(
        "\
program T;
begin
  if true then
    Std.Console.WriteLn('then')
  else
    Std.Console.WriteLn('else')
end.",
    );
    assert_eq!(out.lines, vec!["then"]);
}

#[test]
fn if_false_takes_else() {
    let out = compile_and_run(
        "\
program T;
begin
  if false then
    Std.Console.WriteLn('then')
  else
    Std.Console.WriteLn('else')
end.",
    );
    assert_eq!(out.lines, vec!["else"]);
}
