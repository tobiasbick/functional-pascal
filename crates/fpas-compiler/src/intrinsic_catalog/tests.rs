//! Canonical call coverage for every stable intrinsic identifier.

use super::*;

#[test]
fn stable_intrinsics_have_only_their_canonical_source_route() {
    for intrinsic in Intrinsic::all() {
        // Generic error patterns lower directly to this private IR operation.
        if intrinsic == Intrinsic::Conv(ConvIntrinsic::ErrorMatchesVariant) {
            assert_eq!(resolve("Std.Os.Errors.MatchesVariant", None), None);
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
                    "Std.Ai.Acp.{}.{}{root}",
                    if version == 2 { "V2" } else { "Types" },
                    if decode { "Decode" } else { "Encode" }
                ),
                None,
            )
        }
        Intrinsic::Acp(operation) if operation.connection() => {
            let member = format!("{operation:?}");
            let name = if let Some(member) = member.strip_prefix("Host") {
                format!("Std.Ai.Acp.Host.{member}")
            } else if let Some(member) = member.strip_prefix("V2") {
                format!("Std.Ai.Acp.V2.{member}")
            } else {
                format!("Std.Ai.Acp.{member}")
            };
            (name, None)
        }
        Intrinsic::Conv(ConvIntrinsic::ErrorMessage) => ("Std.Os.Errors.Message".into(), None),
        Intrinsic::Conv(ConvIntrinsic::IntToReal) => ("Std.Math.IntToReal".into(), None),
        Intrinsic::Str(StrIntrinsic::Utf8Encode) => ("Std.Text.Utf8.EncodeBytes".into(), None),
        Intrinsic::Str(StrIntrinsic::Utf8Decode) => ("Std.Text.Utf8.DecodeBytes".into(), None),
        Intrinsic::Str(StrIntrinsic::Utf8Append) => ("Std.Text.Utf8.AppendText".into(), None),
        Intrinsic::Bytes(BytesIntrinsic::JoinChunks) => {
            ("Std.Net.Http.Io.JoinBuffers".into(), None)
        }
        Intrinsic::Bytes(BytesIntrinsic::DecodePercent) => {
            ("Std.Net.Uri.Percent.DecodeBytes".into(), None)
        }
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
            ("Std.Net.Http.Stream.ReserveState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::HasBodyStreamState) => {
            ("Std.Net.Http.Stream.HasState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::LoadBodyStreamState) => {
            ("Std.Net.Http.Stream.LoadState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::StoreBodyStreamState) => {
            ("Std.Net.Http.Stream.StoreState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::ReserveSseDecoderState) => {
            ("Std.Net.Http.Sse.ReserveState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::HasSseDecoderState) => {
            ("Std.Net.Http.Sse.HasState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::LoadSseDecoderState) => {
            ("Std.Net.Http.Sse.LoadState".into(), None)
        }
        Intrinsic::Http(HttpIntrinsic::StoreSseDecoderState) => {
            ("Std.Net.Http.Sse.StoreState".into(), None)
        }
        intrinsic => {
            let debug = format!("{intrinsic:?}");
            let (family, member) = debug
                .split_once('(')
                .expect("intrinsic debug form contains family and member");
            let family = match family {
                "Acp" => "Ai.Acp.Types",
                "Mcp" => "Ai.Mcp",
                "Args" => "Os.Args",
                "Bits" => "Math.Bits",
                "Conv" => "Text.Format",
                "Env" => "Os.Env",
                "Fs" => "Io.Fs",
                "Json" => "Encoding.Json",
                "Net" => "Net.Sockets",
                "Parse" => "Text.Parse",
                "Path" => "Io.Path",
                "Proc" => "Os.Proc",
                "Random" => "Math.Random",
                "Server" => "Net.Server",
                "Toml" => "Encoding.Toml",
                "Tokenizer" => "Ai.Tokenizer",
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
                ("Net.Sockets", "Read") => "ReceiveBytes",
                ("Net.Sockets", "ReadWithCancellation") => "ReceiveBytesWithCancellation",
                ("Net.Sockets", "Write") => "SendBytes",
                ("Net.Sockets", "WriteWithCancellation") => "SendBytesWithCancellation",
                (_, other) => other,
            };
            (format!("Std.{family}.{member}"), None)
        }
    }
}
