//! Failed allocations retain the original text and buffer and their nominal error family.
use super::*;
use crate::{Intrinsic, stdlib::intrinsics::TEST_AGGREGATES};

#[test]
fn encoding_and_appending_report_buffer_allocation_failures_atomically() {
    let original = SharedBytes::from_slice(&[0, 255, 1]).unwrap();
    let capacity = original.capacity();
    for (operation, arguments, requested) in [
        (StrIntrinsic::Utf8Encode, vec![Value::Str("é🙂".into())], 6),
        (
            StrIntrinsic::Utf8Append,
            vec![Value::Bytes(original.clone()), Value::Str("é".into())],
            64,
        ),
    ] {
        SharedBytes::fail_next_allocation();
        let result = crate::stdlib::run_intrinsic_borrowed(
            Intrinsic::Str(operation),
            &arguments,
            SourceLocation::new(1, 1),
            &TEST_AGGREGATES,
        )
        .unwrap()
        .unwrap();
        let Value::ResultError(error) = result else {
            panic!("expected allocation error");
        };
        let Value::Enum(error) = error.as_ref() else {
            panic!("expected nominal buffer error");
        };
        assert_eq!(error.body().layout.type_name, "Std.Bytes.BufferError");
        assert_eq!(error.body().layout.variant, "AllocationFailed");
        assert_eq!(error.body().values, [Value::Integer(requested)]);
        assert_eq!(original.as_ref(), [0, 255, 1]);
        assert_eq!(original.capacity(), capacity);
    }
}
