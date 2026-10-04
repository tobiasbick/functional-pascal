//! String index writes fail during checking, while reads and replacements run.

use super::*;

const INVALID_SOURCE: &str = r#"program StringWrite;
begin
   var Text: string := 'abc';
  Text[0] := 'z';
end program;
"#;

#[test]
fn check_and_run_report_string_index_writes_as_semantic_errors() {
    let cwd = create_temp_dir("string-index-write");
    let main = cwd.join("main.fpas");
    write_text(&main, INVALID_SOURCE);
    for command in ["check", "run"] {
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[command.to_string(), main.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(exit, 1, "{command}: {stderr}");
        assert!(stdout.is_empty(), "{command}: {stdout}");
        assert!(
            stderr.contains(":4:7: error[F2005]: String indices are read-only"),
            "{command}: {stderr}"
        );
        assert!(stderr.contains("whole string"), "{command}: {stderr}");
        assert!(!stderr.contains("F9001"), "{command}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove string index test directory");
}

#[test]
fn json_diagnostic_locates_the_read_only_string_index() {
    let cwd = create_temp_dir("string-index-json");
    let main = cwd.join("main.fpas");
    write_text(&main, INVALID_SOURCE);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "check".to_string(),
            "--diagnostics".to_string(),
            "json".to_string(),
            main.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("remove string index test directory");
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    let records: Vec<serde_json::Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON diagnostic"))
        .collect();
    assert_eq!(records.len(), 1, "{stderr}");
    let record = &records[0];
    assert_eq!(record["code"], "F2005");
    assert_eq!(record["phase"], "sema");
    assert_eq!(record["location"]["start"]["line"], 4);
    assert_eq!(record["location"]["start"]["column"], 7);
    assert!(record["hint"].as_str().unwrap().contains("whole string"));
}

#[test]
fn run_preserves_unicode_reads_and_collection_replacements_with_snapshots() {
    let cwd = create_temp_dir("string-index-read");
    let main = cwd.join("main.fpas");
    write_text(
        &main,
        r#"program StringRead;
uses Std.Console as Console;
type Holder = record Value: string; end record;
begin
   var Text: string := 'old';
   var Items: array of (string) := ['old'];
   var Mapping: dict of (string, string) := ['key': 'old'];
   var Nested: dict of (string, array of (Holder)) := ['key': [Holder(Value := 'old')]];
  const Copy: dict of (string, array of (Holder)) := Nested;
  Text := 'ä😀';
  Items[0] := Text;
  Mapping['key'] := Text;
  Nested['key'][0].Value := Text;
  Console.WriteLn(Text[0]);
  Console.WriteLn(Items[0][1]);
  Console.WriteLn(Mapping['key'][0][0]);
  Console.WriteLn(Copy['key'][0].Value);
  Console.WriteLn(Nested['key'][0].Value);
end program;
"#,
    );
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &["run".to_string(), main.to_string_lossy().into_owned()],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("remove string index test directory");
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "ä\n😀\nä\nold\nä😀\n");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn project_aliases_reject_nested_string_writes_and_allow_element_replacement() {
    let cwd = create_temp_dir("string-index-project");
    let project = cwd.join("app.fpasprj");
    crate::test_support::write_program_fpasprj(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/state.fpas"),
        r#"unit App.State;
public type TextAlias = string;
public type Holder of (T) = record public Value: T; end record;
public type TextHolder = Holder of (TextAlias);
public  var Text: TextAlias := 'abc';
public  var Items: array of (TextHolder) := [TextHolder(Value := 'abc')];
end unit;
"#,
    );
    let main = cwd.join("src/main.fpas");
    for target in ["Store.Text[0]", "Store.Items[0].Value[0]"] {
        write_text(
            &main,
            &format!("program Main; uses App.State as Store; begin {target} := 'z'; end program;"),
        );
        for command in ["check", "run"] {
            let (exit, _, stderr) = support::run_cli_args_and_capture_output(
                &[command.to_string(), project.to_string_lossy().into_owned()],
                &cwd,
            );
            assert_eq!(exit, 1, "{target}, {command}: {stderr}");
            assert!(
                stderr.contains("F2005") && stderr.contains("String indices are read-only"),
                "{target}, {command}: {stderr}"
            );
            assert!(!stderr.contains("F9001"), "{stderr}");
        }
    }
    write_text(
        &main,
        "program Main; uses App.State as Store; uses Std.Console as Console;
        begin Store.Text := 'new'; Store.Items[0].Value := Store.Text;
          Console.WriteLn(Store.Items[0].Value[0]); end program;",
    );
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove string index test project");
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "n\n");
}
