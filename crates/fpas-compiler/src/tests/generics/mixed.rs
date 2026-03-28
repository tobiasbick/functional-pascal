use super::*;

// ═══════════════════════════════════════════════════════════════
// MIXED: generic function + generic type
// ═══════════════════════════════════════════════════════════════

#[test]
fn generic_function_with_generic_record() {
    let out = compile_and_run(
        "\
program GenericMixed;
uses Std.Console;

type
  Wrapper<T> = record
    Value: T;
  end;

function Wrap<T>(V: T): Wrapper of T;
begin
  return record Value := V; end
end;

begin
  var W: Wrapper of integer := Wrap(123);
  WriteLn(W.Value)
end.",
    );
    assert_eq!(out.lines, vec!["123"]);
}

#[test]
fn generic_function_returning_generic_enum() {
    let out = compile_and_run(
        "\
program GenericFuncEnum;
uses Std.Console;

type
  Maybe<T> = enum
    Just(Value: T);
    Nothing;
  end;

function TryMake<T>(V: T; IsOk: boolean): Maybe of T;
begin
  if IsOk then
    return Maybe.Just(V)
  else
    return Maybe.Nothing
end;

begin
  var M: Maybe of string := TryMake('hi', true);
  case M of
    Maybe.Just(V): WriteLn(V);
    Maybe.Nothing: WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["hi"]);
}

#[test]
fn generic_function_with_generic_param_and_concrete() {
    let out = compile_and_run(
        "\
program GenericMixedParams;
uses Std.Console, Std.Conv;

function Describe<T>(Value: T; Label: string): string;
begin
  return Label + ': done'
end;

begin
  WriteLn(Describe(42, 'number'))
end.",
    );
    assert_eq!(out.lines, vec!["number: done"]);
}
