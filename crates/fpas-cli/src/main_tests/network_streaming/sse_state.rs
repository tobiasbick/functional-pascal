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
    source.push_str(
        r#"
public function ReviewRetained(Decoder: Std.Http.Types.SseDecoder): integer;
begin
  var Index: integer := Std.Http.Handles.SseDecoderSlot(Decoder);
  var State: DecoderState := LoadState(Index);
  return Std.Arrays.Length(State.Buffer) + Std.Str.Length(State.Data) +
    Std.Str.Length(State.EventType) + Std.Str.Length(State.LastEventId) + State.EventBytes
end;
"#,
    );
    write_text(&decoder, &source);
    let api = library.join("Std/Http.fpas");
    let mut source = fs::read_to_string(&api).expect("HTTP API source");
    source.push_str(
        r#"
public function ReviewRetained(Decoder: SseDecoder): integer;
begin return Std.Http.Sse.ReviewRetained(Decoder) end;
"#,
    );
    write_text(&api, &source);
    let program = cwd.join("main.fpas");
    write_text(
        &program,
        r#"program RetainedSse;
uses Std.Http, Std.Net.Utf8, Std.Results, Std.Str, Std.Test;
begin
  var Decoder: SseDecoder := Unwrap(CreateSseDecoder(32));
  Unwrap(FeedSse(Decoder, Std.Net.Utf8.Encode('id:old' + #10 + 'data:x' + #10)));
  AssertTrue(ReviewRetained(Decoder) > 0);
  case FeedSse(Decoder, Std.Net.Utf8.Encode(Std.Str.RepeatStr('x', 100000))) of
    Ok(_): Fail('expected size failure'); Error(_): begin end
  end;
  AssertEquals(0, ReviewRetained(Decoder));
  var FinalDecoder: SseDecoder := Unwrap(CreateSseDecoder(32));
  Unwrap(FeedSse(FinalDecoder, [255]));
  case FinishSse(FinalDecoder) of
    Ok(_): Fail('expected UTF-8 failure'); Error(_): begin end
  end;
  AssertEquals(0, ReviewRetained(FinalDecoder))
end.
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
