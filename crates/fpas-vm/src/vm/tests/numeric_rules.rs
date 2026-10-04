//! Integer failures and IEEE behavior across typed, dynamic and immediate opcodes.

use super::*;
use fpas_bytecode::{Constant, Opcode, Value};
use fpas_diagnostics::codes::{
    RUNTIME_DIVISION_BY_ZERO, RUNTIME_MODULO_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

fn binary(opcode: Opcode, left: Constant, right: Constant) -> Result<Value, crate::VmError> {
    let (_, registers, _) = execute(verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(opcode, 2, 0, 1),
            return_unit(),
        ],
        vec![left, right],
        vec!["root", "numeric.fpas"],
        3,
    ))?;
    Ok(registers[2].clone())
}

#[test]
fn typed_and_dynamic_arithmetic_check_both_integer_boundaries() {
    for (typed, dynamic, cases) in [
        (
            Opcode::AddInteger,
            Opcode::AddDynamic,
            [(i64::MAX, 1), (i64::MIN, -1)],
        ),
        (
            Opcode::SubtractInteger,
            Opcode::SubtractDynamic,
            [(i64::MIN, 1), (i64::MAX, -1)],
        ),
        (
            Opcode::MultiplyInteger,
            Opcode::MultiplyDynamic,
            [(i64::MAX, 2), (i64::MIN, -1)],
        ),
    ] {
        for opcode in [typed, dynamic] {
            for (left, right) in cases {
                let error = binary(opcode, Constant::Integer(left), Constant::Integer(right))
                    .expect_err("overflow must fail");
                assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
                assert!(error.message.contains("overflow"), "{error:#?}");
                assert_eq!(error.span.expect("source").line(), 41);
            }
        }
    }
    for opcode in [Opcode::AddInteger, Opcode::AddDynamic] {
        assert_eq!(
            binary(opcode, Constant::Integer(i64::MAX), Constant::Integer(0)).unwrap(),
            Value::Integer(i64::MAX)
        );
    }
    for opcode in [Opcode::MultiplyInteger, Opcode::MultiplyDynamic] {
        assert_eq!(
            binary(opcode, Constant::Integer(i64::MIN), Constant::Integer(1)).unwrap(),
            Value::Integer(i64::MIN)
        );
    }
}

#[test]
fn immediate_arithmetic_and_negation_use_the_same_checked_rules() {
    for (opcode, left, right) in [
        (Opcode::AddIntegerImm, i64::MAX, 1_i16),
        (Opcode::AddIntegerImm, i64::MIN, -1),
        (Opcode::DivideIntegerImm, i64::MIN, -1),
    ] {
        let error = execute(verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abc(opcode, 0, 0, right as u16),
                return_unit(),
            ],
            vec![Constant::Integer(left)],
            vec!["root", "numeric.fpas"],
            1,
        ))
        .unwrap_err();
        assert_eq!(
            error.code, RUNTIME_NUMERIC_DOMAIN_ERROR,
            "{opcode:?}: {error:#?}"
        );
    }
    for opcode in [Opcode::NegateInteger, Opcode::NegateDynamic] {
        let error = execute(verified(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abc(opcode, 0, 0, 0),
                return_unit(),
            ],
            vec![Constant::Integer(i64::MIN)],
            vec!["root", "numeric.fpas"],
            1,
        ))
        .unwrap_err();
        assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
    }
}

#[test]
fn integer_division_and_remainder_preserve_zero_and_overflow_codes() {
    for (opcode, zero_code) in [
        (Opcode::DivideInteger, RUNTIME_DIVISION_BY_ZERO),
        (Opcode::RemainderInteger, RUNTIME_MODULO_BY_ZERO),
    ] {
        assert_eq!(
            binary(opcode, Constant::Integer(1), Constant::Integer(0))
                .unwrap_err()
                .code,
            zero_code
        );
        assert_eq!(
            binary(opcode, Constant::Integer(i64::MIN), Constant::Integer(-1))
                .unwrap_err()
                .code,
            RUNTIME_NUMERIC_DOMAIN_ERROR
        );
    }
    assert_eq!(
        binary(
            Opcode::DivideInteger,
            Constant::Integer(-7),
            Constant::Integer(3)
        )
        .unwrap(),
        Value::Integer(-2)
    );
    assert_eq!(
        binary(
            Opcode::RemainderInteger,
            Constant::Integer(-7),
            Constant::Integer(3)
        )
        .unwrap(),
        Value::Integer(-1)
    );
}

#[test]
fn real_division_preserves_signed_zero_infinity_and_nan() {
    for opcode in [Opcode::DivideReal, Opcode::DivideDynamic] {
        for (left, right, expected) in [
            (1.0_f64, 0.0_f64, f64::INFINITY),
            (1.0, -0.0, f64::NEG_INFINITY),
        ] {
            assert_eq!(
                binary(
                    opcode,
                    Constant::Real(left.to_bits()),
                    Constant::Real(right.to_bits())
                )
                .unwrap(),
                Value::Real(expected)
            );
        }
        let Value::Real(value) = binary(
            opcode,
            Constant::Real(0.0_f64.to_bits()),
            Constant::Real(0.0_f64.to_bits()),
        )
        .unwrap() else {
            panic!("real result");
        };
        assert!(value.is_nan());
    }
    assert_eq!(
        binary(
            Opcode::DivideDynamic,
            Constant::Integer(1),
            Constant::Integer(0)
        )
        .unwrap(),
        Value::Real(f64::INFINITY)
    );
}

#[test]
fn nan_is_unordered_without_suppressing_dynamic_type_errors() {
    for (typed, dynamic) in [
        (Opcode::LessReal, Opcode::LessDynamic),
        (Opcode::LessEqualReal, Opcode::LessEqualDynamic),
        (Opcode::GreaterReal, Opcode::GreaterDynamic),
        (Opcode::GreaterEqualReal, Opcode::GreaterEqualDynamic),
        (Opcode::EqualReal, Opcode::EqualDynamic),
    ] {
        for opcode in [typed, dynamic] {
            for (left, right) in [(f64::NAN, 1.0_f64), (1.0, f64::NAN)] {
                assert_eq!(
                    binary(
                        opcode,
                        Constant::Real(left.to_bits()),
                        Constant::Real(right.to_bits())
                    )
                    .unwrap(),
                    Value::Boolean(false)
                );
            }
        }
    }
    assert_eq!(
        binary(
            Opcode::NotEqualDynamic,
            Constant::Real(f64::NAN.to_bits()),
            Constant::Real(f64::NAN.to_bits())
        )
        .unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        binary(
            Opcode::LessDynamic,
            Constant::Real(f64::NAN.to_bits()),
            Constant::Integer(1)
        )
        .unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        binary(
            Opcode::LessDynamic,
            Constant::Real(f64::NAN.to_bits()),
            Constant::Boolean(true)
        )
        .unwrap_err()
        .code,
        RUNTIME_VM_OPERAND_TYPE_MISMATCH
    );
}
