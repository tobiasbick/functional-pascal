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
        abx(Opcode::LoadConstant, 12, 1),
        abc(Opcode::ShiftLeftInteger, 13, 1, 12),
        abc(Opcode::ShiftRightInteger, 14, 13, 12),
        abc(Opcode::BitAndInteger, 15, 0, 1),
        abc(Opcode::BitOrInteger, 16, 0, 1),
        abc(Opcode::BitXorInteger, 17, 0, 1),
        abc(Opcode::EqualInteger, 18, 1, 14),
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
    assert_eq!(registers[14], Value::Integer(7));
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
    assert_eq!(count, 51);
}

#[test]
fn dynamic_numeric_operations_accept_mixed_integer_and_real_values() {
    let mut code = vec![
        abx(Opcode::LoadConstant, 0, 0),
        abx(Opcode::LoadConstant, 1, 1),
        abc(Opcode::AddDynamic, 2, 0, 1),
        abc(Opcode::SubtractDynamic, 3, 0, 1),
        abc(Opcode::MultiplyDynamic, 4, 0, 1),
        abc(Opcode::DivideDynamic, 5, 0, 1),
        abc(Opcode::NegateDynamic, 6, 0, 0),
        abc(Opcode::LessDynamic, 7, 1, 0),
        abc(Opcode::GreaterEqualDynamic, 8, 0, 1),
        abc(Opcode::NotEqualDynamic, 9, 0, 1),
        abc(Opcode::EqualDynamic, 10, 0, 0),
        abc(Opcode::GreaterDynamic, 11, 0, 1),
        abc(Opcode::LessEqualDynamic, 12, 1, 0),
    ];
    code.push(return_unit());
    let (_, registers, _) = execute(verified(
        code,
        vec![Constant::Integer(7), Constant::Real(2.5_f64.to_bits())],
        vec!["root", "test.fpas"],
        13,
    ))
    .expect("dynamic numeric program should execute");

    assert_eq!(registers[2], Value::Real(9.5));
    assert_eq!(registers[3], Value::Real(4.5));
    assert_eq!(registers[4], Value::Real(17.5));
    assert_eq!(registers[5], Value::Real(2.8));
    assert_eq!(registers[6], Value::Integer(-7));
    assert!(
        registers[7..=12]
            .iter()
            .all(|value| value == &Value::Boolean(true))
    );
}
