use fpas_bytecode::{Intrinsic, MathIntrinsic, SourceLocation, Value};

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

#[test]
fn integer_rounding_rejects_the_first_unrepresentable_boundary() {
    let upper = 9_223_372_036_854_775_808.0_f64;
    for intrinsic in [
        MathIntrinsic::Floor,
        MathIntrinsic::Ceil,
        MathIntrinsic::Round,
        MathIntrinsic::Trunc,
    ] {
        for input in [
            upper,
            (-upper).next_down(),
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let mut stack = vec![Value::Real(input)];
            let error = crate::execute_test_intrinsic(
                Intrinsic::Math(intrinsic),
                &mut stack,
                test_location(),
            )
            .expect_err("rounding must reject values outside the signed integer range");
            assert!(
                error.message.contains("result"),
                "{intrinsic:?}({input}): {error:?}"
            );
        }
        for (input, expected) in [(upper.next_down(), i64::MAX - 1023), (-upper, i64::MIN)] {
            let mut stack = vec![Value::Real(input)];
            crate::execute_test_intrinsic(Intrinsic::Math(intrinsic), &mut stack, test_location())
                .expect("representable boundary must succeed");
            assert_eq!(stack, vec![Value::Integer(expected)]);
        }
    }
}

#[test]
fn abs_reports_overflow_for_min_integer() {
    let mut stack = vec![Value::Integer(i64::MIN)];

    let error = crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Abs),
        &mut stack,
        test_location(),
    )
    .expect_err("Abs must reject integer overflow");

    assert!(error.message.contains("Abs overflow"), "{}", error.message);
}

#[test]
fn floor_rejects_non_finite_values() {
    let mut stack = vec![Value::Real(f64::INFINITY)];

    let error = crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Floor),
        &mut stack,
        test_location(),
    )
    .expect_err("Floor must reject a non-finite result");

    assert!(error.message.contains("Floor result"), "{}", error.message);
}

#[test]
fn trunc_rejects_out_of_range_values() {
    let mut stack = vec![Value::Real(1.0e300)];

    let error = crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Trunc),
        &mut stack,
        test_location(),
    )
    .expect_err("Trunc must reject an out-of-range result");

    assert!(error.message.contains("Trunc result"), "{}", error.message);
}

#[test]
fn floor_ceil_and_trunc_keep_negative_finite_semantics() {
    let mut floor_stack = vec![Value::Real(-3.2)];
    crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Floor),
        &mut floor_stack,
        test_location(),
    )
    .unwrap();
    assert_eq!(floor_stack, vec![Value::Integer(-4)]);

    let mut ceil_stack = vec![Value::Real(-3.2)];
    crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Ceil),
        &mut ceil_stack,
        test_location(),
    )
    .unwrap();
    assert_eq!(ceil_stack, vec![Value::Integer(-3)]);

    let mut trunc_stack = vec![Value::Real(-3.7)];
    crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Trunc),
        &mut trunc_stack,
        test_location(),
    )
    .unwrap();
    assert_eq!(trunc_stack, vec![Value::Integer(-3)]);
}

#[test]
fn round_accepts_regular_finite_values() {
    let mut stack = vec![Value::Real(2.6)];

    crate::execute_test_intrinsic(
        Intrinsic::Math(MathIntrinsic::Round),
        &mut stack,
        test_location(),
    )
    .unwrap();

    assert_eq!(stack, vec![Value::Integer(3)]);
}
