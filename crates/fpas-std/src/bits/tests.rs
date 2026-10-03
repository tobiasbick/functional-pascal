use fpas_bytecode::{BitsIntrinsic as B, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

fn run(op: B, args: &[i64]) -> Result<Vec<Value>, crate::StdError> {
    let mut values = args.iter().copied().map(Value::Integer).collect();
    crate::execute_test_intrinsic(Intrinsic::Bits(op), &mut values, SourceLocation::new(3, 7))?;
    Ok(values)
}

#[test]
fn bit_patterns_include_sign_bit_and_all_bits() {
    for (op, args, expected) in [
        (B::BitAnd, vec![12, 10], 8),
        (B::BitOr, vec![12, 10], 14),
        (B::BitXor, vec![12, 10], 6),
        (B::BitNot, vec![0], -1),
        (B::BitNot, vec![-1], 0),
        (B::BitNot, vec![i64::MIN], i64::MAX),
        (B::BitAnd, vec![i64::MIN, -1], i64::MIN),
        (B::BitOr, vec![i64::MIN, i64::MAX], -1),
        (B::BitXor, vec![-1, i64::MIN], i64::MAX),
    ] {
        assert_eq!(
            run(op, &args).unwrap(),
            vec![Value::Integer(expected)],
            "{op:?} {args:?}"
        );
    }
}

#[test]
fn shifts_accept_every_count_and_zero_fill_negative_patterns() {
    for count in 0..64 {
        assert_eq!(
            run(B::ShiftRight, &[-1, count]).unwrap(),
            vec![Value::Integer((u64::MAX >> count) as i64)]
        );
        assert_eq!(
            run(B::ShiftLeft, &[1, count]).unwrap(),
            vec![Value::Integer((1_u64 << count) as i64)]
        );
    }
    for op in [B::ShiftLeft, B::ShiftRight] {
        for value in [0, -1, i64::MIN, i64::MAX] {
            assert_eq!(run(op, &[value, 0]).unwrap(), vec![Value::Integer(value)]);
        }
    }
    assert_eq!(
        run(B::ShiftLeft, &[i64::MIN, 1]).unwrap(),
        vec![Value::Integer(0)]
    );
    assert_eq!(
        run(B::ShiftRight, &[i64::MIN, 63]).unwrap(),
        vec![Value::Integer(1)]
    );
}

#[test]
fn invalid_counts_fail_even_for_zero_values() {
    for op in [B::ShiftLeft, B::ShiftRight] {
        for count in [i64::MIN, -1, 64, 65, i64::MAX] {
            let error = run(op, &[0, count]).unwrap_err();
            assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
            assert!(error.message.contains(&count.to_string()));
        }
    }
}

#[test]
fn runtime_rejects_wrong_types_and_arity() {
    for args in [
        vec![],
        vec![Value::Integer(1)],
        vec![Value::Integer(1), Value::Boolean(true)],
        vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
    ] {
        let mut args = args;
        assert!(
            crate::execute_test_intrinsic(
                Intrinsic::Bits(B::BitAnd),
                &mut args,
                SourceLocation::new(1, 1)
            )
            .is_err()
        );
    }
    let error = crate::execute_test_intrinsic(
        Intrinsic::Bits(B::BitNot),
        &mut vec![Value::Real(1.0)],
        SourceLocation::new(1, 1),
    )
    .unwrap_err();
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}
