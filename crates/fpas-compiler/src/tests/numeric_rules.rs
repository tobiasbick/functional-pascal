//! Checked arithmetic survives folding, generic specialization and calls.

use super::{assert_succeeds, parse_ok, run_program};
use fpas_bytecode::Opcode;
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_MODULO_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR,
    SEMA_INVALID_STATIC_OPERATION,
};

#[test]
fn static_numeric_failures_are_reported_before_bytecode_generation() {
    for source in [
        "program T; begin const Value := 9223372036854775807 + 1; end program;",
        "program T; begin const Value := [1 div 0] = [0]; end program;",
        "program T; begin const Value := 0 in [1 mod 0]; end program;",
        "program T; uses Std.Math as Math; begin const Value := ([Math.Pi] = [Math.Pi]) and (1 div 0 = 0); end program;",
        "program T; type Data = record Value: integer := 1 div 0; end record; begin const Item := Data(Value := 1); end program;",
    ] {
        let errors = crate::compile(&parse_ok(source)).expect_err("static failure");
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert_eq!(errors[0].code, SEMA_INVALID_STATIC_OPERATION);
    }
}

#[test]
fn builtin_constants_have_identical_static_and_runtime_values() {
    assert_succeeds(
        "program T; uses Std.Math as Math; uses Std.Console as Console; uses Std.Test as Test;
        function Identity(Value: real): real; begin return Value; end function;
        type Data = record Value: real := Math.Pi; end record;
        begin
          const Same := ([Math.Pi] = [Math.Pi]) or (1 div 0 = 0);
          const Different := (Math.Pi < 3.0) and (1 mod 0 = 0);
          const Item := Data();
          Test.AssertTrue(Same); Test.AssertFalse(Different);
          Test.AssertTrue(Item.Value = Identity(Math.Pi));
          Test.AssertTrue(Math.Pi = 3.141592653589793);
          Test.AssertTrue(Console.White + Console.Blink = 143);
          case Console.Font8x8 of when Console.Font8x8: null;
            else Test.Fail('static console constant'); end case;
          begin const Pi := 0.0;
            const Skip := (Pi > 3.0) and (1 div 0 = 0);
            Test.AssertFalse(Skip);
          end;
        end program;",
    );
}

#[test]
fn static_aggregate_guards_match_runtime_values_and_skip_failures() {
    assert_succeeds(
        "program T; uses Std.Test as Test;
        type Data = record Value: integer := 1; end record;
        type Alias = Data;
        const Values: array of (integer) := [1];
        begin
          const No := (Values = [2]) and (1 div 0 = 0);
          const Yes := (['a': 1, 'b': 2] = ['b': 2, 'a': 1]) or (1 mod 0 = 0);
          const Item := Data();
          const Same := (Item = Alias(Value := 1)) or (1 div 0 = 0);
          const Missing := (3 in Values) and (1 div 0 = 0);
          Test.AssertFalse(No); Test.AssertTrue(Yes); Test.AssertTrue(Same);
          Test.AssertFalse(Missing);
          case true of when 1 in Values: null; else Test.Fail('static membership'); end case;
        end program;",
    );
}

#[test]
fn optimizer_retains_invalid_integer_operations_for_runtime_contexts() {
    for (expression, opcodes, expected) in [
        (
            "9223372036854775807 + 1",
            [Opcode::AddInteger, Opcode::AddIntegerImm],
            RUNTIME_NUMERIC_DOMAIN_ERROR,
        ),
        (
            "(-9223372036854775807 - 1) - 1",
            [Opcode::SubtractInteger, Opcode::SubtractInteger],
            RUNTIME_NUMERIC_DOMAIN_ERROR,
        ),
        (
            "9223372036854775807 * 2",
            [Opcode::MultiplyInteger, Opcode::MultiplyInteger],
            RUNTIME_NUMERIC_DOMAIN_ERROR,
        ),
        (
            "1 div 0",
            [Opcode::DivideInteger, Opcode::DivideIntegerImm],
            RUNTIME_DIVISION_BY_ZERO,
        ),
        (
            "1 mod 0",
            [Opcode::RemainderInteger, Opcode::RemainderInteger],
            RUNTIME_MODULO_BY_ZERO,
        ),
    ] {
        let source = format!("program T; begin var Value := {expression}; end program;");
        let program = parse_ok(&source);
        let mut ir = crate::lower(&program).unwrap();
        ir.validate().unwrap();
        crate::optimize::optimize(&mut ir);
        ir.validate().unwrap();
        let optimized = crate::bytecode::compile_program(ir).unwrap();
        assert!(
            optimized
                .executable()
                .code
                .iter()
                .any(|instruction| instruction
                    .opcode()
                    .is_ok_and(|opcode| opcodes.contains(&opcode))),
            "failing operation was discarded: {source}"
        );
        let error = fpas_vm::Vm::new(optimized)
            .run()
            .expect_err("optimized failure");
        assert_eq!(error.code, expected, "{source}: {error:#?}");
    }
}

#[test]
fn ordinary_and_generic_calls_do_not_wrap_integer_results() {
    for heading in [
        "function Calculate(Value: integer): integer;",
        "function Calculate of (T: Numeric)(Value: T): T;",
    ] {
        let source = format!(
            "program T; {heading} begin return Value + Value; end function; begin const ResultValue := Calculate(9223372036854775807); end program;"
        );
        let error = run_program(&source).expect_err("checked call");
        assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR, "{error:#?}");
    }
}

#[test]
fn folded_and_runtime_ieee_results_agree() {
    assert_succeeds("program T; uses Std.Test as Test;
        function Divide(Left: real; Right: real): real; begin return Left / Right; end function;
        function Less of (T: Comparable)(Left: T; Right: T): boolean; begin return Left < Right; end function;
        const Infinite: real := 1 / 0; const Nan: real := 0 / 0;
        begin const RuntimeInfinite := Divide(1.0, 0.0); const RuntimeNan := Divide(0.0, 0.0);
          Test.AssertTrue(Infinite = RuntimeInfinite); Test.AssertTrue(Nan <> Nan);
          Test.AssertTrue(RuntimeNan <> RuntimeNan); Test.AssertFalse(Nan < Infinite);
          Test.AssertFalse(Less(RuntimeNan, RuntimeInfinite));
          Test.AssertTrue(Divide(1.0, -0.0) < 0.0); Test.AssertTrue(0.0 = -0.0);
        end program;");
}
