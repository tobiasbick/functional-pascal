use super::super::*;

// ── language/basics: Constants with various types ──────────────────

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

// ── language/basics: Var block with multiple declarations ──────────

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

// ── language/basics: Immutable variable reassign is error ──────────

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

// ── language/basics: Local variable in function ────────────────────

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

// ── language/basics: Comments are ignored in full programs ─────────

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

// ── language/basics: Type alias used in program ────────────────────

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
