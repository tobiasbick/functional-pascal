//! Execution and bytecode contracts for Boolean short-circuit lowering.

use fpas_bytecode::Opcode;

use super::super::{assert_succeeds, parse_ok, run_program};

#[test]
fn boolean_short_circuit_uses_branches_even_with_constant_operands() {
    for (expression, expected) in [
        ("false and (1 div 0 > 0)", false),
        ("true or (1 div 0 > 0)", true),
    ] {
        let source = format!(
            "program Guarded; begin var Flag: boolean := {expression}; if Flag <> {expected} then panic('result'); end if; end."
        );
        let executable = crate::compile(&parse_ok(&source)).expect("compile guarded expression");
        assert!(executable.executable().code.iter().any(|word| matches!(
            word.opcode(),
            Ok(Opcode::DivideInteger | Opcode::DivideIntegerImm)
        )));
        assert_succeeds(&source);
    }
}

#[test]
fn needed_boolean_operands_and_eager_bits_arguments_still_fail() {
    for (ty, expression) in [
        ("boolean", "true and (1 div 0 > 0)"),
        ("boolean", "false or (1 div 0 > 0)"),
        ("boolean", "false xor (1 div 0 > 0)"),
        ("boolean", "true xor (1 div 0 > 0)"),
        ("integer", "BitAnd(0, 1 div 0)"),
        ("integer", "BitOr(-1, 1 div 0)"),
    ] {
        let source =
            format!("program Needed; uses Std.Bits; begin var Flag: {ty} := {expression}; end.");
        assert!(
            run_program(&source).is_err(),
            "{expression} must evaluate the right operand"
        );
    }
}

#[test]
fn skipped_operands_are_still_type_checked() {
    let program = parse_ok("program Checked; begin var Flag: boolean := false and 123; end.");
    assert!(crate::compile(&program).is_err());
}

#[test]
fn nested_short_circuit_values_survive_calls_loops_and_mutation() {
    assert_succeeds(
        r#"
program BooleanSnapshots;
mutable var Trace: integer := 0;
mutable var Flag: boolean := true;
function Mark(Value: boolean; Digit: integer): boolean;
begin Trace := Trace * 10 + Digit; return Value; end function;
function Change(): boolean;
begin Flag := false; return true; end function;
function Combine(First: boolean; Second: boolean): boolean;
begin return First and Second; end function;
begin
  for I: integer := 1 to 3 do
    Trace := 0;
    if not Combine(Mark(true, 1), Mark(false, 2) or Mark(true, 3)) then panic('call value'); end if;
    if Trace <> 123 then panic('call order'); end if;
    Trace := 0;
    var Values: array of boolean := [Mark(false, 1), Mark(true, 2) and Mark(true, 3)];
    if Values[0] or not Values[1] then panic('array values'); end if;
    if Trace <> 123 then panic('array order'); end if;
    Flag := true;
    if not (Flag and Change()) then panic('left snapshot'); end if;
    if Flag then panic('mutation'); end if;
  end for;
  Trace := 0;
  mutable var Count: integer := 0;
  while (Count < 2) and Mark(true, 1) do Count := Count + 1; end while;
  if Trace <> 11 then panic('while skip'); end if;
end.
"#,
    );
}
