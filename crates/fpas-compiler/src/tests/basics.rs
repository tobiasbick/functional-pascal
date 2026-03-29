use super::*;

#[test]
fn empty_program() {
    let chunk = compile_ok(
        "\
program Empty;
begin
end.",
    );
    assert!(chunk.code.last() == Some(&Op::Halt));
}

#[test]
fn hello_world() {
    let out = compile_and_run(
        "\
program Hello;
begin
  Std.Console.WriteLn('Hello, World!')
end.",
    );
    assert_eq!(out.lines, vec!["Hello, World!"]);
}

#[test]
fn integer_arithmetic() {
    let out = compile_and_run(
        "\
program Arith;
begin
  Std.Console.WriteLn(2 + 3 * 4)
end.",
    );
    // Parser handles precedence: 2 + (3 * 4) = 14
    assert_eq!(out.lines, vec!["14"]);
}

#[test]
fn variable_let_and_print() {
    let out = compile_and_run(
        "\
program VarTest;
begin
  var X: integer := 42;
  Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn mutable_variable_assign() {
    let out = compile_and_run(
        "\
program MutTest;
begin
  mutable var X: integer := 10;
  X := 20;
  Std.Console.WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["20"]);
}

#[test]
fn multiple_prints() {
    let out = compile_and_run(
        "\
program Multi;
begin
  Std.Console.WriteLn(1);
  Std.Console.WriteLn(2);
  Std.Console.WriteLn(3)
end.",
    );
    assert_eq!(out.lines, vec!["1", "2", "3"]);
}

#[test]
fn multiline_string_literal() {
    let out = compile_and_run(
        "\
program MultiLine;
begin
  Std.Console.WriteLn('Roses are red
Violets are blue')
end.",
    );
    assert_eq!(out.lines, vec!["Roses are red\nViolets are blue"]);
}

#[test]
fn character_code_concatenation() {
    let out = compile_and_run(
        "\
program CharCodes;
begin
  Std.Console.WriteLn('Hello'#13#10'World')
end.",
    );
    assert_eq!(out.lines, vec!["Hello\r\nWorld"]);
}

#[test]
fn xor_and_shift_operators() {
    let out = compile_and_run(
        "\
program Bitwise;
begin
  Std.Console.WriteLn(6 xor 3);
  Std.Console.WriteLn(1 shl 4);
  Std.Console.WriteLn(16 shr 2)
end.",
    );
    assert_eq!(out.lines, vec!["5", "16", "4"]);
}

#[test]
fn const_expression_must_be_compile_time_known() {
    let err = compile_err(
        "\
program ConstExpr;

function FortyTwo(): integer;
begin
  return 42
end;

const
  Answer: integer := FortyTwo();

begin
end.",
    );
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION
    );
}
