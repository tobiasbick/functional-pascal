//! Tail calls and overlapping argument windows keep call semantics.

use fpas_bytecode::Opcode;

use super::{assert_succeeds, parse_ok, run_program};

fn opcodes(source: &str) -> Vec<Opcode> {
    let executable = crate::compile(&parse_ok(source)).expect("program compiles");
    executable
        .executable()
        .code
        .iter()
        .filter_map(|word| word.opcode().ok())
        .collect()
}

#[test]
fn tail_recursion_runs_deeper_than_the_call_stack_limit() {
    let source = "program Deep; function Count(N: integer; Acc: integer): integer; begin if N = 0 then return Acc; return Count(N - 1, Acc + 1) end; begin if Count(100000, 0) <> 100000 then panic('wrong') end.";
    assert!(opcodes(source).contains(&Opcode::TailCall));
    assert_succeeds(source);
}

#[test]
fn non_tail_recursion_still_reports_the_call_stack_limit() {
    let source = "program Deep; function Count(N: integer): integer; begin if N = 0 then return 0; return 1 + Count(N - 1) end; begin if Count(100000) <> 100000 then panic('wrong') end.";
    let error = run_program(source).expect_err("non-tail recursion must overflow");
    assert!(
        error.message.contains("Call stack overflow"),
        "{}",
        error.message
    );
}

#[test]
fn unit_tail_calls_return_to_the_original_caller() {
    let source = "program UnitTail; uses Std.Console; procedure Leaf(N: integer); begin WriteLn(N) end; procedure Forward(N: integer); begin Leaf(N + 1) end; begin Forward(1); WriteLn(3) end.";
    assert!(opcodes(source).contains(&Opcode::TailCall));
    assert_succeeds(source);
}

#[test]
fn value_returning_callees_are_not_tail_called_from_procedures() {
    let source = "program Mixed; function Value(N: integer): integer; begin return N end; procedure Discard(N: integer); begin Value(N) end; begin Discard(1); var Done: boolean := true end.";
    assert!(!opcodes(source).contains(&Opcode::TailCall));
    assert_succeeds(source);
}

#[test]
fn multi_argument_calls_keep_caller_state_and_array_values() {
    let source = "program Windows; uses Std.Arrays;
function Combine(A: integer; B: integer; C: integer): integer; begin return A * 100 + B * 10 + C end;
function Grow(Values: array of integer; Extra: integer): array of integer;
begin
  mutable var Local: array of integer := Values;
  Std.Arrays.Push(Local, Extra);
  return Local
end;
begin
  var Keep: integer := 7;
  var Original: array of integer := [1, 2];
  var Grown: array of integer := Grow(Original, 3);
  if Combine(1, 2, 3) + Combine(4, 5, 6) <> 579 then panic('arguments');
  if Keep <> 7 then panic('caller register');
  if Std.Arrays.Length(Original) <> 2 then panic('caller array changed');
  if Std.Arrays.Length(Grown) <> 3 then panic('callee array')
end.";
    assert_succeeds(source);
}
