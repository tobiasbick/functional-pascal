//! Successful parsing must not copy its borrowed input text.
#![allow(clippy::expect_used, reason = "valid intrinsic fixtures")]
use crate::intrinsics::{TEST_AGGREGATES, run_intrinsic_borrowed};
use fpas_bytecode::{ConvIntrinsic, Intrinsic, ParseIntrinsic, SourceLocation, Value};

#[test]
fn parsing_allocations_do_not_grow_with_borrowed_input_padding() {
    for (intrinsic, text) in [
        (Intrinsic::Parse(ParseIntrinsic::TryInt), "42"),
        (Intrinsic::Parse(ParseIntrinsic::TryReal), "3.5"),
        (Intrinsic::Parse(ParseIntrinsic::TryBool), "true"),
        (Intrinsic::Conv(ConvIntrinsic::StrToInt), "42"),
        (Intrinsic::Conv(ConvIntrinsic::StrToReal), "3.5"),
        (Intrinsic::Conv(ConvIntrinsic::StrToBool), "true"),
    ] {
        let measure = |text: String| {
            let args = [Value::Str(text.into())];
            let mut result = None;
            let allocations = allocation_counter::measure(|| {
                result = Some(run_intrinsic_borrowed(
                    intrinsic,
                    &args,
                    SourceLocation::new(1, 1),
                    &TEST_AGGREGATES,
                ));
            });
            (
                allocations,
                result.expect("called intrinsic").expect("successful parse"),
            )
        };
        let (short, result) = measure(text.to_owned());
        let (padded, padded_result) =
            measure(format!("{}{text}{}", " ".repeat(65536), " ".repeat(65536)));
        assert_eq!(result, padded_result);
        assert_eq!(short.count_total, padded.count_total, "{intrinsic:?}");
        assert_eq!(short.bytes_total, padded.bytes_total, "{intrinsic:?}");
        println!(
            "{intrinsic:?}: allocations={}, bytes={}",
            short.count_total, short.bytes_total
        );
    }
}
