use fpas_bytecode::{Constant, Opcode, Value};

use super::*;

#[test]
fn typed_scalar_families_execute_with_aliasing_destinations() {
    let constants = vec![
        Constant::Integer(7),
        Constant::Integer(3),
        Constant::Real(7.5_f64.to_bits()),
        Constant::Real(2.5_f64.to_bits()),
        Constant::Boolean(true),
        Constant::Boolean(false),
        Constant::String(fpas_bytecode::StringId::new(2)),
        Constant::String(fpas_bytecode::StringId::new(3)),
    ];
    let mut code = vec![
        abx(Opcode::LoadConstant, 0, 0),
        abx(Opcode::LoadConstant, 1, 1),
        abx(Opcode::LoadConstant, 2, 2),
        abx(Opcode::LoadConstant, 3, 3),
        abx(Opcode::LoadConstant, 4, 4),
        abx(Opcode::LoadConstant, 5, 5),
        abx(Opcode::LoadConstant, 6, 6),
        abx(Opcode::LoadConstant, 7, 7),
        abc(Opcode::AddInteger, 0, 0, 1),
        abc(Opcode::SubtractInteger, 1, 0, 1),
        abc(Opcode::MultiplyInteger, 8, 0, 1),
        abc(Opcode::DivideInteger, 9, 8, 0),
        abc(Opcode::RemainderInteger, 10, 8, 1),
        abc(Opcode::NegateInteger, 11, 1, 0),
        abc(Opcode::EqualInteger, 18, 1, 1),
        abc(Opcode::LessInteger, 19, 1, 0),
        abc(Opcode::GreaterEqualInteger, 20, 0, 1),
        abc(Opcode::AddReal, 2, 2, 3),
        abc(Opcode::SubtractReal, 3, 2, 3),
        abc(Opcode::MultiplyReal, 21, 2, 3),
        abc(Opcode::DivideReal, 22, 21, 2),
        abc(Opcode::NegateReal, 23, 3, 0),
        abc(Opcode::LessReal, 24, 3, 2),
        abc(Opcode::GreaterEqualReal, 25, 2, 3),
        abc(Opcode::ConcatString, 6, 6, 7),
        abc(Opcode::EqualString, 26, 6, 6),
        abc(Opcode::GreaterString, 27, 6, 7),
        abc(Opcode::AndBoolean, 28, 4, 5),
        abc(Opcode::OrBoolean, 29, 4, 5),
        abc(Opcode::NotBoolean, 30, 5, 0),
        abc(Opcode::EqualBoolean, 31, 4, 30),
        abc(Opcode::IntegerToReal, 32, 1, 0),
        abc(Opcode::NotEqualInteger, 33, 0, 1),
        abc(Opcode::GreaterInteger, 34, 0, 1),
        abc(Opcode::LessEqualInteger, 35, 1, 0),
        abc(Opcode::EqualReal, 36, 2, 2),
        abc(Opcode::NotEqualReal, 37, 2, 3),
        abc(Opcode::GreaterReal, 38, 2, 3),
        abc(Opcode::LessEqualReal, 39, 3, 2),
        abc(Opcode::NotEqualString, 40, 6, 7),
        abc(Opcode::LessString, 41, 6, 7),
        abc(Opcode::LessEqualString, 42, 6, 7),
        abc(Opcode::GreaterEqualString, 43, 6, 6),
        abc(Opcode::NotEqualBoolean, 44, 4, 5),
    ];
    code.push(return_unit());

    let (_, registers, count) = execute(verified(
        code,
        constants,
        vec!["root", "test.fpas", "ab", "cd"],
        45,
    ))
    .expect("typed scalar program should execute");

    assert_eq!(registers[0], Value::Integer(10));
    assert_eq!(registers[1], Value::Integer(7));
    assert_eq!(registers[8], Value::Integer(70));
    assert_eq!(registers[9], Value::Integer(7));
    assert_eq!(registers[10], Value::Integer(0));
    assert_eq!(registers[11], Value::Integer(-7));
    assert_eq!(registers[18], Value::Boolean(true));
    assert_eq!(registers[19], Value::Boolean(true));
    assert_eq!(registers[20], Value::Boolean(true));
    assert_eq!(registers[2], Value::Real(10.0));
    assert_eq!(registers[3], Value::Real(7.5));
    assert_eq!(registers[21], Value::Real(75.0));
    assert_eq!(registers[22], Value::Real(7.5));
    assert_eq!(registers[23], Value::Real(-7.5));
    assert_eq!(registers[24], Value::Boolean(true));
    assert_eq!(registers[25], Value::Boolean(true));
    assert_eq!(registers[6].to_string(), "abcd");
    assert_eq!(registers[26], Value::Boolean(true));
    assert_eq!(registers[27], Value::Boolean(false));
    assert_eq!(registers[28], Value::Boolean(false));
    assert_eq!(registers[29], Value::Boolean(true));
    assert_eq!(registers[30], Value::Boolean(true));
    assert_eq!(registers[31], Value::Boolean(true));
    assert_eq!(registers[32], Value::Real(7.0));
    assert!(
        registers[33..=44]
            .iter()
            .all(|value| value == &Value::Boolean(true))
    );
    assert_eq!(count, 45);
}

#[test]
fn dynamic_numeric_operations_reject_mixed_integer_and_real_values() {
    for opcode in [
        Opcode::AddDynamic,
        Opcode::SubtractDynamic,
        Opcode::MultiplyDynamic,
        Opcode::DivideDynamic,
        Opcode::LessDynamic,
        Opcode::GreaterEqualDynamic,
        Opcode::EqualDynamic,
        Opcode::NotEqualDynamic,
        Opcode::GreaterDynamic,
        Opcode::LessEqualDynamic,
    ] {
        for (left, right) in [(0, 1), (1, 0)] {
            let error = execute(verified(
                vec![
                    abx(Opcode::LoadConstant, 0, 0),
                    abx(Opcode::LoadConstant, 1, 1),
                    abc(opcode, 2, left, right),
                    return_unit(),
                ],
                vec![Constant::Integer(7), Constant::Real(2.5_f64.to_bits())],
                vec!["root", "test.fpas"],
                3,
            ))
            .expect_err("mixed numeric operands must fail");
            assert_eq!(
                error.code,
                fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "{opcode:?}"
            );
        }
    }
}

#[test]
fn prepared_string_constants_stay_unchanged_after_concatenation() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::ConcatString, 0, 0, 0),
            abx(Opcode::LoadConstant, 1, 0),
            return_unit(),
        ],
        vec![Constant::String(fpas_bytecode::StringId::new(2))],
        vec!["root", "test.fpas", "ab"],
        2,
    );
    assert_eq!(
        executable.string_constant(0).map(|value| &**value),
        Some("ab")
    );
    let (_, registers, _) = execute(executable).expect("string constants execute");
    assert_eq!(registers[0], Value::Str("abab".into()));
    assert_eq!(registers[1], Value::Str("ab".into()));
}

#[test]
fn typed_real_division_retains_zero_diagnostic() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::DivideReal, 2, 0, 1),
            return_unit(),
        ],
        vec![
            Constant::Real(1.0_f64.to_bits()),
            Constant::Real(0.0_f64.to_bits()),
        ],
        vec!["root", "test.fpas"],
        3,
    );
    let error = execute(executable).expect_err("division by zero must fail");
    assert!(error.message.contains("Division by zero"));
}

#[test]
fn typed_real_ordering_retains_nan_diagnostic() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::LessReal, 2, 0, 1),
            return_unit(),
        ],
        vec![
            Constant::Real(f64::NAN.to_bits()),
            Constant::Real(1.0_f64.to_bits()),
        ],
        vec!["root", "test.fpas"],
        3,
    );
    let error = execute(executable).expect_err("NaN ordering must fail");
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR
    );
}

#[test]
fn signed_integer_immediates_execute_and_preserve_division_errors() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::AddIntegerImm, 0, 0, (-3_i16) as u16),
            abc(Opcode::DivideIntegerImm, 1, 0, (-2_i16) as u16),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        vec!["root", "test.fpas"],
        2,
    );
    let (_, registers, _) = execute(executable).expect("signed immediates execute");
    assert_eq!(registers[0], Value::Integer(4));
    assert_eq!(registers[1], Value::Integer(-2));

    let zero = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::DivideIntegerImm, 1, 0, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        vec!["root", "test.fpas"],
        2,
    );
    let error = execute(zero).expect_err("zero divisor must fail");
    assert!(error.message.contains("Division by zero"));
}

#[test]
fn consuming_concatenation_reuses_temporaries_without_touching_shared_strings() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::Move, 1, 0, 0),
            abx(Opcode::LoadConstant, 2, 1),
            abc_aux(Opcode::ConcatString, 3, 1, 2, 1),
            abc_aux(Opcode::ConcatString, 0, 3, 2, 1),
            return_unit(),
        ],
        vec![
            Constant::String(fpas_bytecode::StringId::new(2)),
            Constant::String(fpas_bytecode::StringId::new(3)),
        ],
        vec!["root", "test.fpas", "ab", "!"],
        4,
    );
    let prepared = executable
        .string_constant(0)
        .cloned()
        .expect("prepared string");
    let (_, registers, _) = execute(executable).expect("concatenation runs");
    assert_eq!(registers[0], Value::Str("ab!!".into()));
    assert_eq!(registers[1], Value::Unit);
    assert_eq!(registers[3], Value::Unit);
    assert_eq!(prepared.as_ref(), "ab");
}

#[test]
fn consuming_move_transfers_the_value_and_clears_the_source() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc_aux(Opcode::Move, 1, 0, 0, 1),
            abc(Opcode::Move, 2, 1, 0),
            return_unit(),
        ],
        vec![Constant::String(fpas_bytecode::StringId::new(2))],
        vec!["root", "test.fpas", "moved"],
        3,
    );
    let mut worker =
        crate::vm::worker::Worker::new(std::sync::Arc::new(executable)).expect("worker");
    worker.dispatch_one().expect("load");
    worker.dispatch_one().expect("consuming move");
    assert_eq!(worker.registers[0], Value::Unit);
    assert!(!worker.register_is_initialized(0));
    assert_eq!(worker.registers[1], Value::Str("moved".into()));
    worker.dispatch_one().expect("copying move");
    assert_eq!(worker.registers[1], worker.registers[2]);
    assert!(worker.register_is_initialized(1));
}

#[test]
fn function_values_share_the_prepared_name() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 0),
            return_unit(),
        ],
        vec![Constant::Function {
            function: fpas_bytecode::FunctionId::new(0),
            task_bound: false,
        }],
        vec!["root", "test.fpas"],
        2,
    );
    let (_, registers, _) = execute(executable).expect("function constants load");
    let (Value::Function(first), Value::Function(second)) = (&registers[0], &registers[1]) else {
        panic!("expected function values");
    };
    assert_eq!(&*first.name, "root");
    assert!(std::sync::Arc::ptr_eq(&first.name, &second.name));
}

#[test]
fn typed_real_arithmetic_rejects_non_finite_operands_and_results() {
    for (left, right) in [(f64::MAX, 2.0), (f64::INFINITY, 1.0), (1.0, f64::NAN)] {
        let executable = verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abx(Opcode::LoadConstant, 1, 1),
                abc(Opcode::MultiplyReal, 2, 0, 1),
                return_unit(),
            ],
            vec![
                Constant::Real(left.to_bits()),
                Constant::Real(right.to_bits()),
            ],
            vec!["root", "test.fpas"],
            3,
        );
        let error = execute(executable).expect_err("non-finite real arithmetic must fail");
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR
        );
    }
}

#[test]
fn direct_integer_arithmetic_matches_checked_operations_at_boundaries() {
    use crate::vm::execute::integer::direct_integer_binary;
    use crate::vm::value_ops::{BinaryOperation, integer_binary};

    let values = [
        i64::MIN,
        i64::MIN + 1,
        -2,
        -1,
        0,
        1,
        2,
        i64::MAX - 1,
        i64::MAX,
    ];
    let operations = [
        BinaryOperation::Add,
        BinaryOperation::Subtract,
        BinaryOperation::Multiply,
        BinaryOperation::IntegerDivide,
        BinaryOperation::Modulo,
        BinaryOperation::Equal,
        BinaryOperation::NotEqual,
        BinaryOperation::Less,
        BinaryOperation::LessEqual,
        BinaryOperation::Greater,
        BinaryOperation::GreaterEqual,
    ];
    for operation in operations {
        for left in values {
            for right in values {
                if let Some(direct) = direct_integer_binary(operation, left, right) {
                    let checked = integer_binary(operation, left, right).ok();
                    assert_eq!(Some(direct), checked, "{operation:?} {left} {right}");
                }
            }
        }
    }
}
