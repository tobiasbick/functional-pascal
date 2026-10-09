//! Native string bounds hints through text/JSON diagnostics and executable corrections.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`,
//! `docs/pascal/language/types/string/format-chars.md`,
//! `docs/pascal/language/types/string/edit.md`.

use super::{create_temp_dir, fs, support, write_text};
use serde_json::Value;

fn cli(command: &str, format: &str, source: &str) -> (i32, String, String) {
    let dir = create_temp_dir("string-bounds-diagnostics");
    let path = dir.join("bounds.fpas");
    write_text(&path, source);
    let result = support::run_cli_args_and_capture_output(
        &[
            command.into(),
            "--diagnostics".into(),
            format.into(),
            path.to_string_lossy().into_owned(),
        ],
        &dir,
    );
    fs::remove_dir_all(dir).expect("remove string-bounds fixture");
    result
}

fn source(text: &str, call: &str) -> String {
    format!("program Bounds;\nbegin\n  const S: string := '{text}';\n  discard S.{call};\nend.\n")
}

fn runtime_record(text: &str, call: &str) -> Value {
    let (exit, stdout, stderr) = cli("run", "json", &source(text, call));
    assert_eq!(exit, 2, "{call}\n{stderr}");
    assert!(stdout.is_empty(), "{call}\n{stdout}");
    let record: Value = serde_json::from_str(stderr.trim()).expect("one JSON diagnostic");
    assert_eq!(record["code"], "FP5021", "{call}");
    assert_eq!(record["phase"], "runtime", "{call}");
    assert_eq!(record["severity"], "error", "{call}");
    assert_eq!(record["location"]["start"]["line"], 4, "{call}");
    assert!(
        record["source"]
            .as_str()
            .expect("source")
            .ends_with("bounds.fpas")
    );
    record
}

#[test]
fn nonempty_character_string_bounds_json_hints_use_native_length() {
    for index in [-1, 2, i64::MAX] {
        for call in [
            format!("CharAt({index})"),
            format!("SetCharAt({index}, 'z')"),
        ] {
            let record = runtime_record("é😀", &call);
            assert_eq!(
                record["hint"],
                "Ensure the index is within 0..S.Length()-1."
            );
            assert!(
                record["message"]
                    .as_str()
                    .expect("message")
                    .contains("length 2")
            );
        }
    }
}

#[test]
fn empty_character_string_bounds_json_hints_recommend_is_empty() {
    for index in [-1, 0, 1] {
        for call in [
            format!("CharAt({index})"),
            format!("SetCharAt({index}, 'z')"),
        ] {
            let record = runtime_record("", &call);
            assert_eq!(
                record["hint"],
                "An empty string has no valid character index. Check S.IsEmpty() before calling this operation."
            );
        }
    }
}

#[test]
fn insertion_string_bounds_json_hints_allow_the_string_end() {
    for (text, length) in [("", 0), ("é😀", 2)] {
        for index in [-1, length + 1, i64::MAX] {
            let record = runtime_record(text, &format!("Insert({index}, 'z')"));
            assert_eq!(record["hint"], "Ensure the index is within 0..S.Length().");
            assert_eq!(
                record["message"],
                format!("Insert index {index} out of range (length {length})")
            );
        }
    }
}

#[test]
fn string_bounds_text_hints_match_the_native_and_empty_string_corrections() {
    for (text, call, hint) in [
        (
            "é😀",
            "CharAt(2)",
            "Ensure the index is within 0..S.Length()-1.",
        ),
        (
            "é😀",
            "SetCharAt(2, 'z')",
            "Ensure the index is within 0..S.Length()-1.",
        ),
        (
            "",
            "CharAt(0)",
            "An empty string has no valid character index. Check S.IsEmpty() before calling this operation.",
        ),
        (
            "",
            "SetCharAt(0, 'z')",
            "An empty string has no valid character index. Check S.IsEmpty() before calling this operation.",
        ),
        (
            "",
            "Insert(1, 'z')",
            "Ensure the index is within 0..S.Length().",
        ),
    ] {
        let (exit, stdout, stderr) = cli("run", "text", &source(text, call));
        assert_eq!(exit, 2, "{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert!(stderr.contains("error[FP5021]"), "{stderr}");
        assert!(stderr.contains(&format!("help: {hint}")), "{stderr}");
        assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
    }
}

#[test]
fn string_bounds_hint_calls_compile_and_preserve_valid_scalar_boundaries() {
    let source = "\
program Corrections;
begin
  const S: string := 'é😀';
  const Empty: string := '';
  if S.CharAt(S.Length()-1) <> '😀' then
    panic('last scalar');
  end if;
  if S.SetCharAt(S.Length()-1, 'z') <> 'éz' then
    panic('replace last scalar');
  end if;
  if not Empty.IsEmpty() then
    discard Empty.CharAt(0);
    discard Empty.SetCharAt(0, 'z');
  end if;
  if Empty.Insert(Empty.Length(), 'z') <> 'z' then
    panic('empty insertion');
  end if;
  if S.Insert(S.Length(), 'z') <> 'é😀z' then
    panic('append after scalars');
  end if;
end.
";
    for command in ["check", "run"] {
        let (exit, stdout, stderr) = cli(command, "json", source);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "", ""),
            "{command}"
        );
    }
}
