//! `fpas doc` text and JSON output, project lookup and the no-match contract.

use super::*;

use crate::main_tests::support::run_cli_args_and_capture_output;

fn repository_library() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../lib")
        .to_string_lossy()
        .into_owned()
}

fn doc(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let mut argv = vec![String::from("doc")];
    argv.extend(args.iter().map(|arg| (*arg).to_owned()));
    argv.extend([String::from("--std-lib"), repository_library()]);
    run_cli_args_and_capture_output(&argv, cwd)
}

#[test]
fn doc_prints_signature_documentation_location_and_handbook() {
    let cwd = create_temp_dir("doc-text");
    let (code, stdout, stderr) = doc(&cwd, &["Std.Fs.ReadText"]);

    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        stdout.starts_with(
            "Std.Fs.ReadText (function)\n  function ReadText(Path: string): result of (string, string)\n\n  Reads UTF-8 text.\n"
        ),
        "{stdout}"
    );
    assert!(stdout.contains("Fs.fpas:"), "{stdout}");
    assert!(
        stdout.ends_with("  Handbook: docs/pascal/std/host/fs.md\n"),
        "{stdout}"
    );
}

#[test]
fn doc_reports_built_in_operations_as_json() {
    let cwd = create_temp_dir("doc-json");
    let (code, stdout, stderr) = doc(&cwd, &["--report", "json", "string.TrimLeft"]);

    assert_eq!(code, 0, "{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON document");
    assert_eq!(report["query"], "string.TrimLeft");
    assert_eq!(report["suggestions"], serde_json::json!([]));
    let entry = &report["matches"][0];
    assert_eq!(entry["name"], "string.TrimLeft");
    assert_eq!(entry["kind"], "operation");
    assert_eq!(entry["signature"], "string.TrimLeft(): string");
    assert_eq!(entry["location"], serde_json::Value::Null);
    assert_eq!(
        entry["handbook"],
        "docs/pascal/language/types/string/README.md"
    );
}

#[test]
fn doc_finds_project_declarations_through_an_explicit_path() {
    let cwd = create_temp_dir("doc-project");
    write_text(
        &cwd.join("app/app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    write_text(
        &cwd.join("app/src/main.fpas"),
        "program App;\n\n// Doubles a value.\nfunction Twice(Value: integer): integer;\nbegin\n  return Value * 2;\nend function;\n\nbegin\n  discard Twice(2);\nend.\n",
    );

    let (code, stdout, stderr) = doc(&cwd, &["--report", "json", "Twice", "app/app.fpasprj"]);

    assert_eq!(code, 0, "{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON document");
    let entry = &report["matches"][0];
    assert_eq!(entry["name"], "App.Twice");
    assert_eq!(entry["documentation"], "Doubles a value.");
    assert_eq!(
        entry["location"],
        serde_json::json!({"path": "app/src/main.fpas", "line": 4, "column": 10})
    );
}

#[test]
fn doc_without_a_match_exits_one_with_suggestions() {
    let cwd = create_temp_dir("doc-missing");

    let (code, stdout, stderr) = doc(&cwd, &["Fs.ReadTxt"]);
    assert_eq!(code, 1);
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("error[FP4143]"), "{stderr}");
    assert!(stderr.contains("`Std.Fs.ReadText`"), "{stderr}");

    let (code, stdout, _) = doc(&cwd, &["--report", "json", "Fs.ReadTxt"]);
    assert_eq!(code, 1);
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON document");
    assert_eq!(report["matches"], serde_json::json!([]));
    assert_eq!(report["suggestions"][0], "Std.Fs.ReadText");
}

#[test]
fn doc_rejects_missing_names_unknown_options_and_reports() {
    let cwd = create_temp_dir("doc-arguments");
    for (args, expected) in [
        (vec![], "Missing name to look up."),
        (vec!["A", "B", "C"], "takes one name and at most one path"),
        (vec!["--verbose", "A"], "Unknown option `--verbose`"),
        (
            vec!["--report", "xml", "A"],
            "Unsupported report format `xml`",
        ),
    ] {
        let (code, _, stderr) = doc(&cwd, &args);
        assert_eq!(code, 1, "{args:?}");
        assert!(stderr.contains("error[FP4137]"), "{stderr}");
        assert!(stderr.contains(expected), "{args:?}: {stderr}");
    }
}

#[test]
fn doc_help_lists_name_forms_and_examples() {
    let cwd = create_temp_dir("doc-help");
    let (code, stdout, _) =
        run_cli_args_and_capture_output(&[String::from("doc"), String::from("--help")], &cwd);
    assert_eq!(code, 0);
    assert!(stdout.contains("fpas doc <name> [<path>]"), "{stdout}");
    assert!(stdout.contains("fpas doc string.TrimLeft"), "{stdout}");
}
