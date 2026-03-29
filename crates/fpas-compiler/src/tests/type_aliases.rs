/// Tests for type aliases (non-generic).
///
/// **Documentation:** [docs/pascal/05-types.md](docs/pascal/05-types.md)
use super::*;

// ═══════════════════════════════════════════════════════════════
// POSITIVE — scalar type aliases
// ═══════════════════════════════════════════════════════════════

#[test]
fn scalar_type_alias_integer() {
    let out = compile_and_run(
        "\
program AliasInt;
uses Std.Console;
type UserId = integer;
begin
  var Id: UserId := 42;
  WriteLn(Id)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn scalar_type_alias_string() {
    let out = compile_and_run(
        "\
program AliasStr;
uses Std.Console;
type UserName = string;
begin
  var N: UserName := 'Alice';
  WriteLn(N)
end.",
    );
    assert_eq!(out.lines, vec!["Alice"]);
}

#[test]
fn type_alias_used_in_function_param() {
    let out = compile_and_run(
        "\
program AliasFuncParam;
uses Std.Console;
type Score = integer;

function Double(S: Score): Score;
begin
  return S * 2
end;

begin
  WriteLn(Double(21))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn type_alias_interchangeable_with_base_type() {
    let out = compile_and_run(
        "\
program AliasInterchange;
uses Std.Console;
type Age = integer;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var X: Age := 10;
  WriteLn(Add(X, 5))
end.",
    );
    assert_eq!(out.lines, vec!["15"]);
}

#[test]
fn callback_type_alias() {
    let out = compile_and_run(
        "\
program CallbackAlias;
uses Std.Console;
type Callback = function(Value: integer): boolean;

function IsPositive(Value: integer): boolean;
begin
  return Value > 0
end;

begin
  var Cb: Callback := IsPositive;
  WriteLn(Cb(5));
  WriteLn(Cb(-1))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

// ═══════════════════════════════════════════════════════════════
// NEGATIVE
// ═══════════════════════════════════════════════════════════════

#[test]
fn type_alias_unknown_base_type() {
    let err = compile_err(
        "\
program AliasBad;
type Foo = Nonexistent;
begin
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE);
}

#[test]
fn type_alias_array() {
    let out = compile_and_run(
        "\
program AliasArr;
uses Std.Console, Std.Array, Std.Conv;
type IntList = array of integer;
begin
  var L: IntList := [10, 20, 30];
  WriteLn(IntToStr(Std.Array.Length(L)));
  WriteLn(L[1])
end.",
    );
    assert_eq!(out.lines, vec!["3", "20"]);
}

#[test]
fn type_alias_record() {
    let out = compile_and_run(
        "\
program AliasRec;
uses Std.Console;
type Coord = record X: integer; Y: integer; end;
type Position = Coord;
begin
  var P: Position := record X := 5; Y := 10; end;
  WriteLn(P.X);
  WriteLn(P.Y)
end.",
    );
    assert_eq!(out.lines, vec!["5", "10"]);
}

#[test]
fn type_alias_chain() {
    let out = compile_and_run(
        "\
program AliasChain;
uses Std.Console;
type Base = integer;
type Middle = Base;
type Top = Middle;
begin
  var X: Top := 42;
  WriteLn(X)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn multiple_type_block_declarations() {
    let out = compile_and_run(
        "\
program MultiType;
uses Std.Console;
type
  UserId = integer;
  UserName = string;
begin
  var Id: UserId := 1;
  var Name: UserName := 'Alice';
  WriteLn(Id);
  WriteLn(Name)
end.",
    );
    assert_eq!(out.lines, vec!["1", "Alice"]);
}
