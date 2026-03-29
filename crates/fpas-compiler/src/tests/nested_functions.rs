/// Tests for nested functions with lexical scope access.
///
/// **Documentation:** [docs/pascal/04-functions.md](docs/pascal/04-functions.md)
use super::*;

#[test]
fn nested_function_reads_parent_param() {
    let out = compile_and_run(
        "\
program NestedParam;

function Outer(x: integer): integer;
  function Inner(): integer;
  begin
    return x * 2
  end;
begin
  return Inner()
end;

begin
  Std.Console.WriteLn(Outer(21))
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn nested_function_reads_parent_local() {
    let out = compile_and_run(
        "\
program NestedLocal;

function Outer(n: integer): string;
  function IsPositive(): boolean;
  begin
    return n > 0
  end;
begin
  if IsPositive() then
    return 'pos'
  else
    return 'neg'
end;

begin
  Std.Console.WriteLn(Outer(5));
  Std.Console.WriteLn(Outer(-3))
end.",
    );
    assert_eq!(out.lines, vec!["pos", "neg"]);
}

#[test]
fn deeply_nested_function() {
    let out = compile_and_run(
        "\
program DeepNested;

function A(x: integer): integer;
  function B(): integer;
    function C(): integer;
    begin
      return x + 100
    end;
  begin
    return C()
  end;
begin
  return B()
end;

begin
  Std.Console.WriteLn(A(7))
end.",
    );
    assert_eq!(out.lines, vec!["107"]);
}

#[test]
fn nested_function_own_params_and_capture() {
    let out = compile_and_run(
        "\
program NestedMixed;

function Outer(base: integer): integer;
  function Add(n: integer): integer;
  begin
    return base + n
  end;
begin
  return Add(8)
end;

begin
  Std.Console.WriteLn(Outer(100))
end.",
    );
    assert_eq!(out.lines, vec!["108"]);
}

#[test]
fn nested_procedure_captures_parent() {
    let out = compile_and_run(
        "\
program NestedProc;

procedure Outer(msg: string);
  procedure Inner();
  begin
    Std.Console.WriteLn(msg)
  end;
begin
  Inner()
end;

begin
  Outer('hello from inner')
end.",
    );
    assert_eq!(out.lines, vec!["hello from inner"]);
}

// ═══════════════════════════════════════════════════════════════
// EDGE CASES
// ═══════════════════════════════════════════════════════════════

#[test]
fn multiple_nested_functions_at_same_level() {
    let out = compile_and_run(
        "\
program MultiNested;

function Compute(X: integer): integer;
  function Double(N: integer): integer;
  begin
    return N * 2
  end;

  function Inc(N: integer): integer;
  begin
    return N + 1
  end;
begin
  return Inc(Double(X))
end;

begin
  Std.Console.WriteLn(Compute(5))
end.",
    );
    assert_eq!(out.lines, vec!["11"]);
}

#[test]
fn nested_function_shadows_outer_param() {
    let out = compile_and_run(
        "\
program NestedShadow;

function Outer(X: integer): integer;
  function Inner(X: integer): integer;
  begin
    return X * 10
  end;
begin
  return Inner(X + 1)
end;

begin
  Std.Console.WriteLn(Outer(3))
end.",
    );
    assert_eq!(out.lines, vec!["40"]);
}

#[test]
fn nested_function_with_own_local_vars() {
    let out = compile_and_run(
        "\
program NestedLocals;
uses Std.Console;

function Outer(X: integer): integer;
  function Inner(): integer;
  begin
    var Local: integer := X * 3;
    return Local + 1
  end;
begin
  return Inner()
end;

begin
  WriteLn(Outer(10))
end.",
    );
    assert_eq!(out.lines, vec!["31"]);
}
