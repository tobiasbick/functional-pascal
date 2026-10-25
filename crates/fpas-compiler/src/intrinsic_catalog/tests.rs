//! Canonical call coverage for every stable intrinsic identifier.

use super::*;

#[test]
fn stable_intrinsics_have_only_their_canonical_source_route() {
    for intrinsic in Intrinsic::all() {
        // Generic error patterns lower directly to this private IR operation.
        if intrinsic == Intrinsic::Conv(ConvIntrinsic::ErrorMatchesVariant) {
            assert_eq!(resolve("Std.Errors.MatchesVariant", None), None);
            continue;
        }
        let (name, argument_type) = canonical_test_call(intrinsic);
        assert_eq!(
            resolve(&name, argument_type.as_ref()),
            Some(intrinsic),
            "canonical catalog did not resolve {intrinsic:?} through {name}"
        );
    }
}

fn canonical_test_call(intrinsic: Intrinsic) -> (String, Option<Ty>) {
    match intrinsic {
        Intrinsic::Acp(operation) if operation.codec().is_some() => {
            let (version, decode, root) = operation.codec().expect("payload codec");
            (
                format!(
                    "Std.Acp.{}.{}{root}",
                    if version == 2 { "V2" } else { "Types" },
                    if decode { "Decode" } else { "Encode" }
                ),
                None,
            )
        }
        Intrinsic::Conv(ConvIntrinsic::ErrorMessage) => ("Std.Errors.Message".into(), None),
        Intrinsic::Str(StrIntrinsic::Utf8Encode) => ("Std.Net.Utf8.EncodeBytes".into(), None),
        Intrinsic::Str(StrIntrinsic::Repeat) => ("Std.Str.RepeatStr".into(), None),
        Intrinsic::Test(TestIntrinsic::AssertEqualsInteger) => {
            ("Std.Test.AssertEquals".into(), Some(Ty::Integer))
        }
        Intrinsic::Test(TestIntrinsic::AssertEqualsBoolean) => {
            ("Std.Test.AssertEquals".into(), Some(Ty::Boolean))
        }
        Intrinsic::Test(TestIntrinsic::AssertEqualsString) => {
            ("Std.Test.AssertEquals".into(), Some(Ty::String))
        }
        Intrinsic::Test(TestIntrinsic::AssertEqualsReal) => {
            ("Std.Test.AssertEquals".into(), Some(Ty::Real))
        }
        Intrinsic::Http(HttpIntrinsic::ReserveBodyStreamState) => {
            ("Std.Http.Stream.ReserveState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::HasBodyStreamState) => {
            ("Std.Http.Stream.HasState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::LoadBodyStreamState) => {
            ("Std.Http.Stream.LoadState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::StoreBodyStreamState) => {
            ("Std.Http.Stream.StoreState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::ReserveSseDecoderState) => {
            ("Std.Http.Sse.ReserveState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::HasSseDecoderState) => {
            ("Std.Http.Sse.HasState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::LoadSseDecoderState) => {
            ("Std.Http.Sse.LoadState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::StoreSseDecoderState) => {
            ("Std.Http.Sse.StoreState".into(), None)
        }
        intrinsic => {
            let debug = format!("{intrinsic:?}");
            let (family, member) = debug
                .split_once('(')
                .expect("intrinsic debug form contains family and member");
            let family = match family {
                "Acp" => "Acp.Types",
                "Array" => "Arrays",
                "Dict" => "Dictionaries",
                "Result" => "Results",
                "Option" => "Options",
                "Task" => "Tasks",
                other => other,
            };
            let member = member.trim_end_matches(')');
            let member = match (family, member) {
                ("Console", "Read") => "ReadText",
                ("Console", "Write") => "WriteText",
                ("Net", "Read") => "ReceiveBytes",
                ("Net", "ReadWithCancellation") => "ReceiveBytesWithCancellation",
                ("Net", "Write") => "SendBytes",
                ("Net", "WriteWithCancellation") => "SendBytesWithCancellation",
                (_, other) => other,
            };
            (format!("Std.{family}.{member}"), None)
        }
    }
}
