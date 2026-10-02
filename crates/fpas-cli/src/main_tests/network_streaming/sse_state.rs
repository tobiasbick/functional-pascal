//! Inspect production decoder state through a fixture-only source accessor.
use super::super::*;
use std::path::Path;

fn copy_sources(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("fixture directory");
    for entry in fs::read_dir(source).expect("stdlib sources") {
        let entry = entry.expect("source entry");
        let path = entry.path();
        let destination = target.join(entry.file_name());
        if path.is_dir() {
            copy_sources(&path, &destination);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("fpas" | "fpasprj")
        ) {
            fs::copy(path, destination).expect("copy source");
        }
    }
}

#[test]
fn source_review_sse_failure_releases_retained_input() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace");
    let cwd = create_temp_dir("sse-retained-state");
    let library = cwd.join("lib");
    copy_sources(&root.join("lib"), &library);
    let decoder = library.join("Std/Http/Sse.fpas");
    let mut source = fs::read_to_string(&decoder).expect("decoder source");
    let closer = source.rfind("end unit;").expect("unit closer");
    source.insert_str(
        closer,
        r#"
public function ReviewRetained(Decoder: Types.SseDecoder): integer;
begin
  var Index: integer := Handles.SseDecoderSlot(Decoder);
  var State: DecoderState := LoadState(Index);
  return Arrays.Length(State.Buffer) + Str.Length(State.Data) +
    Str.Length(State.EventType) + Str.Length(State.LastEventId) + State.EventBytes;
end function;
"#,
    );
    write_text(&decoder, &source);
    let api = library.join("Std/Http.fpas");
    let mut source = fs::read_to_string(&api).expect("HTTP API source");
    let closer = source.rfind("end unit;").expect("unit closer");
    source.insert_str(
        closer,
        r#"
public function ReviewRetained(Decoder: SseDecoder): integer;
begin return Sse.ReviewRetained(Decoder); end function;
"#,
    );
    write_text(&api, &source);
    let program = cwd.join("main.fpas");
    write_text(
        &program,
        r#"program RetainedSse;
uses Std.Http as Http; uses Std.Net.Utf8 as Utf8; uses Std.Results as Results; uses Std.Str as Str; uses Std.Test as Test;
begin
  var Decoder: Http.SseDecoder := Results.Unwrap(Http.CreateSseDecoder(32));
  Results.Unwrap(Http.FeedSse(Decoder, Utf8.Encode('id:old' + #10 + 'data:x' + #10)));
  Test.AssertTrue(Http.ReviewRetained(Decoder) > 0);
  case Http.FeedSse(Decoder, Utf8.Encode(Str.RepeatStr('x', 100000))) of
    when Ok(_): Test.Fail('expected size failure'); when Error(_): begin null; end;
  end case;
  Test.AssertEquals(0, Http.ReviewRetained(Decoder));
  var FinalDecoder: Http.SseDecoder := Results.Unwrap(Http.CreateSseDecoder(32));
  Results.Unwrap(Http.FeedSse(FinalDecoder, [255]));
  case Http.FinishSse(FinalDecoder) of
    when Ok(_): Test.Fail('expected UTF-8 failure'); when Error(_): begin null; end;
  end case;
  Test.AssertEquals(0, Http.ReviewRetained(FinalDecoder));
end program;
"#,
    );
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "--std-lib".into(),
            library.to_string_lossy().into_owned(),
            program.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(cwd).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
}
