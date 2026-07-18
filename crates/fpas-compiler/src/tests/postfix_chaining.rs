use super::*;

#[test]
fn create_next_value_chain() {
    let out = compile_and_run(
        "\
program PostfixChain;
type Num = record
  V: integer;
  function Next(Self: Num): Num;
  begin
    return record V := Self.V + 1; end
  end;
  static function Create(V: integer): Num;
  begin
    return record V := V; end
  end;
end;
begin
  Std.Console.WriteLn(Num.Create(10).Next().V)
end.",
    );
    assert_eq!(out.lines, vec!["11"]);
}

#[test]
fn create_array_index_chain() {
    let out = compile_and_run(
        "\
program PostfixIndex;
function CreateArray(): array of integer;
begin
  return [7, 8, 9]
end;
begin
  Std.Console.WriteLn(CreateArray()[1])
end.",
    );
    assert_eq!(out.lines, vec!["8"]);
}

#[test]
fn base_call_executes_exactly_once() {
    let out = compile_and_run(
        "\
program PostfixOnce;
mutable var Hits: integer := 0;
type Holder = record
  Value: integer;
end;
function Build(): Holder;
begin
  Hits := Hits + 1;
  return record Value := Hits; end
end;
begin
  Std.Console.WriteLn(Build().Value);
  Std.Console.WriteLn(Hits)
end.",
    );
    assert_eq!(out.lines, vec!["1", "1"]);
}

#[test]
fn method_arguments_evaluated_once_in_source_order() {
    let out = compile_and_run(
        "\
program PostfixArgs;
mutable var Log: string := '';
type Acc = record
  Value: integer;
  function Add(Self: Acc; N: integer): Acc;
  begin
    return record Value := Self.Value + N; end
  end;
end;
function Mark(Label: string; N: integer): integer;
begin
  Log := Log + Label;
  return N
end;
function Start(): Acc;
begin
  return record Value := 0; end
end;
begin
  Std.Console.WriteLn(Start().Add(Mark('A', 1)).Add(Mark('B', 2)).Value);
  Std.Console.WriteLn(Log)
end.",
    );
    assert_eq!(out.lines, vec!["3", "AB"]);
}

#[test]
fn chained_field_access_uses_field_get() {
    let chunk = compile_ok(
        "\
program PostfixFields;
type Inner = record X: integer; end;
type Outer = record Nested: Inner; end;
function Make(): Outer;
begin
  return record Nested := record X := 42; end; end
end;
begin
  Std.Console.WriteLn(Make().Nested.X)
end.",
    );
    assert!(
        chunk.code().iter().any(|op| matches!(op, Op::FieldGet(_))),
        "expected FieldGet in chunk: {:?}",
        chunk.code()
    );
    let out = compile_and_run(
        "\
program PostfixFieldsRun;
type Inner = record X: integer; end;
type Outer = record Nested: Inner; end;
function Make(): Outer;
begin
  return record Nested := record X := 42; end; end
end;
begin
  Std.Console.WriteLn(Make().Nested.X)
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn decision_examples_compile_and_run() {
    let out = compile_and_run(
        "\
program PostfixDecision;
type Box = record
  Value: integer;
  function Transform(Self: Box; Factor: integer): Box;
  begin
    return record Value := Self.Value * Factor; end
  end;
  static function Create(): Box;
  begin
    return record Value := 3; end
  end;
end;
type Item = record
  Items: array of integer;
  static function Create(): Item;
  begin
    return record Items := [5, 6]; end
  end;
end;
begin
  Std.Console.WriteLn(Box.Create().Value);
  Std.Console.WriteLn(Box.Create().Transform(2).Value);
  Std.Console.WriteLn(Item.Create().Items[0]);
  Std.Console.WriteLn((Box.Create()).Value)
end.",
    );
    assert_eq!(out.lines, vec!["3", "6", "5", "3"]);
}

#[test]
fn method_arity_overflow_is_diagnosed() {
    let params: String = (1..=255)
        .map(|i| format!("A{i}: integer"))
        .collect::<Vec<_>>()
        .join("; ");
    let args: String = (1..=255).map(|_| "1").collect::<Vec<_>>().join(", ");
    let err = compile_err(&format!(
        "\
program PostfixArity;
type R = record
  X: integer;
  function Many(Self: R; {params}): integer;
  begin
    return Self.X
  end;
end;
function Make(): R; begin return record X := 1; end end;
begin
  Std.Console.WriteLn(Make().Many({args}))
end."
    ));
    assert_eq!(
        err.code,
        fpas_diagnostics::codes::COMPILE_BYTECODE_OPERAND_OVERFLOW,
        "unexpected error: {}",
        err.message
    );
}

#[test]
fn generic_call_results_continue_through_fields() {
    let out = compile_and_run(
        "\
program PostfixGenericResults;
type Value = record Number: integer; end;
type Box = record
  Number: integer;
  function Map<T>(Self: Box; Fn: function(X: integer): T): T;
  begin
    return Fn(Self.Number)
  end;
end;
type Factory = record
  static function Identity<T>(Input: T): T;
  begin
    return Input
  end;
end;
function Identity<T>(Input: T): T;
begin
  return Input
end;
function CreateValue(): Value;
begin
  return record Number := 5; end
end;
function CreateBox(): Box;
begin
  return record Number := 7; end
end;
function Wrap(N: integer): Value;
begin
  return record Number := N; end
end;
begin
  Std.Console.WriteLn(Identity(CreateValue()).Number);
  Std.Console.WriteLn(Factory.Identity(CreateValue()).Number);
  Std.Console.WriteLn(CreateBox().Map(Wrap).Number)
end.",
    );
    assert_eq!(out.lines, vec!["5", "5", "7"]);
}

#[test]
fn final_procedure_method_runs_as_statement() {
    let out = compile_and_run(
        "\
program PostfixProcedureStatement;
mutable var Hits: integer := 0;
type Handle = record
  procedure Destroy(Self: Handle);
  begin
    Hits := Hits + 1
  end;
  static function Create(): Handle;
  begin
    return record end
  end;
end;
begin
  Handle.Create().Destroy();
  Std.Console.WriteLn(Hits)
end.",
    );
    assert_eq!(out.lines, vec!["1"]);
}
