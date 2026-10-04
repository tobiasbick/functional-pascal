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
    let source = r#"program Deep; function Count(N: integer; Acc: integer): integer; begin if N = 0 then return Acc; end if; return Count(N - 1, Acc + 1); end function; begin if Count(100000, 0) <> 100000 then panic('wrong'); end if; end program;"#;
    assert!(opcodes(source).contains(&Opcode::TailCall));
    assert_succeeds(source);
}

#[test]
fn non_tail_recursion_still_reports_the_call_stack_limit() {
    let source = r#"program Deep; function Count(N: integer): integer; begin if N = 0 then return 0; end if; return 1 + Count(N - 1); end function; begin if Count(100000) <> 100000 then panic('wrong'); end if; end program;"#;
    let error = run_program(source).expect_err("non-tail recursion must overflow");
    assert!(
        error.message.contains("Call stack overflow"),
        "{}",
        error.message
    );
}

#[test]
fn unit_tail_calls_return_to_the_original_caller() {
    let source = r#"program UnitTail;  uses Std.Console as Console; procedure Leaf(N: integer); begin Console.WriteLn(N); end procedure; procedure Forward(N: integer); begin Leaf(N + 1); end procedure; begin Forward(1); Console.WriteLn(3); end program;"#;
    assert!(opcodes(source).contains(&Opcode::TailCall));
    assert_succeeds(source);
}

#[test]
fn value_returning_callees_are_not_tail_called_from_procedures() {
    let source = r#"program Mixed; function Value(N: integer): integer; begin return N; end function; procedure IgnoreValue(N: integer); begin discard Value(N); end procedure; begin IgnoreValue(1); const Done: boolean := true; end program;"#;
    assert!(!opcodes(source).contains(&Opcode::TailCall));
    assert_succeeds(source);
}

#[test]
fn multi_argument_calls_keep_caller_state_and_array_values() {
    let source = r#"program Windows;  uses Std.Arrays as Arrays;
function Combine(A: integer; B: integer; C: integer): integer; begin return A * 100 + B * 10 + C; end function;
function Grow(Values: array of (integer); Extra: integer): array of (integer);
begin
   var Local: array of (integer) := Values;
  Arrays.Push(var Local, Extra);
  return Local;
end function;
begin
  const Keep: integer := 7;
  const Original: array of (integer) := [1, 2];
  const Grown: array of (integer) := Grow(Original, 3);
  if Combine(1, 2, 3) + Combine(4, 5, 6) <> 579 then panic('arguments'); end if;
  if Keep <> 7 then panic('caller register'); end if;
  if Arrays.Length(Original) <> 2 then panic('caller array changed'); end if;
  if Arrays.Length(Grown) <> 3 then panic('callee array'); end if;
end program;"#;
    assert_succeeds(source);
}

#[test]
fn tail_calls_through_function_values_reuse_the_frame() {
    let source = r#"program ValueTail; function Countdown(N: integer): integer; begin if N = 0 then return 0; end if; const Next: function(N: integer): integer := Countdown; return Next(N - 1); end function; begin if Countdown(100000) <> 0 then panic('wrong'); end if; end program;"#;
    assert!(opcodes(source).contains(&Opcode::TailCallValue));
    assert_succeeds(source);
}
