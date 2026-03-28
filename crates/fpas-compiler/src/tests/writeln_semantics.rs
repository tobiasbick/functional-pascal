use super::*;

#[test]
fn writeln_multiple_args_single_captured_line() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn('X', 'Y')
end.",
    );
    assert_eq!(out.lines, vec!["XY"]);
}

#[test]
fn writeln_no_args_emits_blank_line() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn('A');
  WriteLn();
  WriteLn('B')
end.",
    );
    assert_eq!(out.lines, vec!["A", "", "B"]);
}

#[test]
fn writeln_three_values_concatenate() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  WriteLn(1, 2, 3)
end.",
    );
    assert_eq!(out.lines, vec!["123"]);
}
