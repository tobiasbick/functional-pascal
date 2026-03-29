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

// ── 02-basics.md: Number Literals ───────────────────────────────

#[test]
fn hex_literal_value() {
    let out = compile_and_run(
        "\
program HexLit;
uses Std.Console;
begin
  WriteLn($FF);
  WriteLn($FF_FF)
end.",
    );
    assert_eq!(out.lines, vec!["255", "65535"]);
}

#[test]
fn underscore_literal() {
    let out = compile_and_run(
        "\
program UnderscoreLit;
uses Std.Console;
begin
  WriteLn(1_000_000)
end.",
    );
    assert_eq!(out.lines, vec!["1000000"]);
}

#[test]
fn scientific_notation_value() {
    let out = compile_and_run(
        "\
program SciNote;
uses Std.Console;
begin
  WriteLn(1.5e2);
  WriteLn(3.0E-1)
end.",
    );
    assert_eq!(out.lines, vec!["150", "0.3"]);
}

// ── 02-basics.md: String Concatenation ──────────────────────────

#[test]
fn string_concat_with_plus() {
    let out = compile_and_run(
        "\
program StrConcat;
uses Std.Console;
begin
  var Full: string := 'Hello' + ' ' + 'World';
  WriteLn(Full)
end.",
    );
    assert_eq!(out.lines, vec!["Hello World"]);
}

#[test]
fn escaped_apostrophe_in_output() {
    let out = compile_and_run(
        "\
program EscApos;
uses Std.Console;
begin
  WriteLn('It''s Pascal')
end.",
    );
    assert_eq!(out.lines, vec!["It's Pascal"]);
}

// ── 02-basics.md: div and mod ───────────────────────────────────

#[test]
fn div_and_mod_execution() {
    let out = compile_and_run(
        "\
program DivMod;
uses Std.Console;
begin
  WriteLn(10 div 3);
  WriteLn(10 mod 3)
end.",
    );
    assert_eq!(out.lines, vec!["3", "1"]);
}

// ── 02-basics.md: Constants with various types ──────────────────

#[test]
fn const_real_and_string_and_boolean() {
    let out = compile_and_run(
        "\
program ConstTypes;
uses Std.Console;
const
  Pi: real := 3.14;
  Greeting: string := 'Hello';
  Flag: boolean := true;
begin
  WriteLn(Pi);
  WriteLn(Greeting);
  WriteLn(Flag)
end.",
    );
    assert_eq!(out.lines, vec!["3.14", "Hello", "true"]);
}

// ── 02-basics.md: Var block with multiple declarations ──────────

#[test]
fn var_block_multiple_declarations() {
    let out = compile_and_run(
        "\
program VarBlock;
uses Std.Console;
var
  Name: string := 'Alice';
  Age: integer := 30;
begin
  WriteLn(Name);
  WriteLn(Age)
end.",
    );
    assert_eq!(out.lines, vec!["Alice", "30"]);
}

// ── 02-basics.md: Immutable variable reassign is error ──────────

#[test]
fn immutable_var_reassign_is_compile_error() {
    let err = compile_err(
        "\
program ImmutableErr;
var
  X: integer := 10;
begin
  X := 20
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT);
}

// ── 02-basics.md: Local variable in function ────────────────────

#[test]
fn local_var_inside_function() {
    let out = compile_and_run(
        "\
program LocalVar;
uses Std.Console;

function FullName(First: string; Last: string): string;
begin
  var Space: string := ' ';
  return First + Space + Last
end;

begin
  WriteLn(FullName('John', 'Doe'))
end.",
    );
    assert_eq!(out.lines, vec!["John Doe"]);
}

// ── 02-basics.md: Comments are ignored in full programs ─────────

#[test]
fn comments_ignored_in_program() {
    let out = compile_and_run(
        "\
program Comments;
uses Std.Console;
{ This is a brace comment }
(* This is a paren-star comment *)
// This is a line comment
begin
  WriteLn('A'); { inline }
  WriteLn('B'); (* inline *)
  WriteLn('C')  // inline
end.",
    );
    assert_eq!(out.lines, vec!["A", "B", "C"]);
}

// ── 02-basics.md: Type alias used in program ────────────────────

#[test]
fn type_alias_end_to_end() {
    let out = compile_and_run(
        "\
program TypeAlias;
uses Std.Console;
type
  Name = string;
  Age = integer;
begin
  var N: Name := 'Alice';
  var A: Age := 30;
  WriteLn(N);
  WriteLn(A)
end.",
    );
    assert_eq!(out.lines, vec!["Alice", "30"]);
}

// ── 02-basics.md: Comparison operators ──────────────────────────

#[test]
fn comparison_operators_all() {
    let out = compile_and_run(
        "\
program CmpOps;
uses Std.Console;
begin
  WriteLn(1 = 1);
  WriteLn(1 <> 2);
  WriteLn(1 < 2);
  WriteLn(2 > 1);
  WriteLn(1 <= 1);
  WriteLn(1 >= 1)
end.",
    );
    assert_eq!(
        out.lines,
        vec!["true", "true", "true", "true", "true", "true"]
    );
}

// ── 02-basics.md: Logical operators on booleans ─────────────────

#[test]
fn logical_operators_full() {
    let out = compile_and_run(
        "\
program LogOps;
uses Std.Console;
begin
  WriteLn(true and true);
  WriteLn(true and false);
  WriteLn(false or true);
  WriteLn(false or false);
  WriteLn(not true);
  WriteLn(true xor false);
  WriteLn(true xor true)
end.",
    );
    assert_eq!(
        out.lines,
        vec!["true", "false", "true", "false", "false", "true", "false"]
    );
}

// ── 02-basics.md: Bitwise operators on integers ─────────────────

#[test]
fn bitwise_and_or_not() {
    let out = compile_and_run(
        "\
program BitwiseOps;
uses Std.Console;
begin
  WriteLn(12 and 10);
  WriteLn(12 or 3);
  WriteLn(not 0)
end.",
    );
    assert_eq!(out.lines, vec!["8", "15", "-1"]);
}
