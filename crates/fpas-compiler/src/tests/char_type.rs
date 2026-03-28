use super::*;

#[test]
fn char_variable_print() {
    let out = compile_and_run(
        "\
program CharVar;
var
  C: char := 'A';
begin
  Std.Console.WriteLn(C)
end.",
    );
    assert_eq!(out.lines, vec!["A"]);
}

#[test]
fn char_code_syntax() {
    let out = compile_and_run(
        "\
program CharCode;
var
  Tab: char := #9;
  Letter: char := #65;
begin
  Std.Console.WriteLn(Letter)
end.",
    );
    assert_eq!(out.lines, vec!["A"]);
}

#[test]
fn char_comparison() {
    let out = compile_and_run(
        "\
program CharCmp;
var
  A: char := 'A';
  B: char := 'B';
begin
  if A < B then
    Std.Console.WriteLn('yes')
  else
    Std.Console.WriteLn('no')
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}

#[test]
fn char_equality() {
    let out = compile_and_run(
        "\
program CharEq;
var
  C: char := 'X';
begin
  if C = 'X' then
    Std.Console.WriteLn('match')
  else
    Std.Console.WriteLn('no match')
end.",
    );
    assert_eq!(out.lines, vec!["match"]);
}

#[test]
fn char_in_case() {
    let out = compile_and_run(
        "\
program CharCase;
var
  Grade: char := 'B';
begin
  case Grade of
    'A': Std.Console.WriteLn('excellent');
    'B': Std.Console.WriteLn('good');
    'C': Std.Console.WriteLn('fair');
  else
    Std.Console.WriteLn('unknown')
  end
end.",
    );
    assert_eq!(out.lines, vec!["good"]);
}

#[test]
fn char_passed_to_function() {
    let out = compile_and_run(
        "\
program CharFunc;

function IsVowel(c: char): boolean;
begin
  if (c = 'A') or (c = 'E') or (c = 'I') or (c = 'O') or (c = 'U') then
    return true
  else
    return false
end;

begin
  if IsVowel('E') then
    Std.Console.WriteLn('vowel')
  else
    Std.Console.WriteLn('consonant')
end.",
    );
    assert_eq!(out.lines, vec!["vowel"]);
}

#[test]
fn char_assigned_to_string() {
    let out = compile_and_run(
        "\
program CharToStr;
var
  S: string := 'Z';
begin
  Std.Console.WriteLn(S)
end.",
    );
    assert_eq!(out.lines, vec!["Z"]);
}
