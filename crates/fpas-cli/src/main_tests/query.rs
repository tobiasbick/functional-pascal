//! JSON query contracts at the actual CLI dispatch boundary.

mod rename;

use super::*;
use crate::main_tests::support::run_cli_args_and_capture_output;
use serde_json::{Value, json};

fn query(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let mut argv = vec!["query".to_owned()];
    argv.extend(args.iter().map(|arg| (*arg).to_owned()));
    if !args.is_empty() {
        argv.extend([
            "--std-lib".into(),
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../lib")
                .to_string_lossy()
                .into_owned(),
        ]);
    }
    run_cli_args_and_capture_output(&argv, cwd)
}

fn report(cwd: &Path, args: &[&str]) -> Value {
    let (code, stdout, stderr) = query(cwd, args);
    assert_eq!(code, 0, "{args:?}: {stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    serde_json::from_str(&stdout).expect("exactly one JSON document")
}

#[test]
fn query_refs_and_def_follow_aliases_case_and_shadowing_without_writing() {
    let cwd = create_temp_dir("query-project");
    write_text(
        &cwd.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let main = "program App;\nuses Demo.Math as M;\nbegin\n  const Total := M.Answer();\n  discard total;\n  begin\n    const Total := 'local';\n    discard TOTAL;\n  end;\n  discard 'Total M.Answer'; // TOTAL M.Answer\n  discard TOTAL;\nend.\n";
    let unit = "unit Demo.Math;\npublic function Answer(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n";
    write_text(&cwd.join("src/main.fpas"), main);
    write_text(&cwd.join("src/math.fpas"), unit);

    let result = report(&cwd, &["refs", "src/main.fpas:4:9"]);
    assert_eq!(result["schemaVersion"], 1);
    assert_eq!(result["query"], "refs");
    let matches = result["matches"].as_array().expect("matches");
    assert_eq!(matches.len(), 2, "{result}");
    assert_eq!(
        matches[0]["range"],
        json!({"start": {"line": 5, "column": 11}, "end": {"line": 5, "column": 16}})
    );
    assert_eq!(matches[1]["range"]["start"]["line"], 11);
    assert_eq!(matches[0]["is_declaration"], false);
    let path = matches[0]["path"].as_str().expect("path");
    assert!(Path::new(path).is_absolute(), "{path}");

    let result = report(
        &cwd,
        &["refs", "src/main.fpas:5:11", "--include-declaration"],
    );
    assert_eq!(result["matches"].as_array().expect("matches").len(), 3);
    assert_eq!(result["matches"][0]["is_declaration"], true);
    let result = report(&cwd, &["def", "src/main.fpas:8:13"]);
    assert_eq!(
        result["matches"][0]["range"]["start"],
        json!({"line": 7, "column": 11})
    );

    let result = report(
        &cwd,
        &["def", "src/main.fpas:4:20", "--project", "app.fpasprj"],
    );
    assert!(
        result["matches"][0]["path"]
            .as_str()
            .expect("unit path")
            .ends_with("/src/math.fpas")
    );
    assert_eq!(
        result["matches"][0]["range"]["start"],
        json!({"line": 2, "column": 17})
    );
    let result = report(
        &cwd,
        &["refs", "src/math.fpas:2:17", "--project", "app.fpasprj"],
    );
    assert_eq!(result["matches"].as_array().expect("matches").len(), 1);
    assert_eq!(
        result["matches"][0]["range"]["start"],
        json!({"line": 4, "column": 20})
    );

    assert_eq!(fs::read_to_string(cwd.join("src/main.fpas")).unwrap(), main);
    assert_eq!(fs::read_to_string(cwd.join("src/math.fpas")).unwrap(), unit);
    assert_eq!(
        fs::read_dir(&cwd).unwrap().count(),
        2,
        "no project artifacts"
    );
    assert_eq!(
        fs::read_dir(cwd.join("src")).unwrap().count(),
        2,
        "no unit artifacts"
    );
}

#[test]
fn query_type_uses_unicode_scalar_columns_crlf_and_absolute_paths() {
    let cwd = create_temp_dir("query-unicode");
    write_text(
        &cwd.join("main.fpas"),
        "program T;\r\nbegin\r\n  discard '😀€'; discard 1 + 2;\r\nend.\r\n",
    );
    let path = format!("{}:3:26", cwd.join("main.fpas").display());
    let result = report(&cwd, &["type", &path]);
    assert_eq!(
        result["matches"],
        json!([{"type": "integer", "range": {
            "start": {"line": 3, "column": 25}, "end": {"line": 3, "column": 30}
        }}])
    );
    assert_eq!(result["position"]["column"], 26);
    let result = report(&cwd, &["type", "main.fpas:3:12"]);
    assert_eq!(result["matches"][0]["type"], "string");
    assert_eq!(
        report(&cwd, &["def", "main.fpas:3:12"])["matches"],
        json!([])
    );
    assert_eq!(
        report(&cwd, &["type", "main.fpas:4:1"])["matches"],
        json!([])
    );
}

#[test]
fn query_rejects_invalid_arguments_positions_sources_and_contexts() {
    let cwd = create_temp_dir("query-errors");
    write_text(&cwd.join("main.fpas"), "program T; begin discard 1; end.");
    for (args, message) in [
        (vec![], "Missing query kind"),
        (vec!["unknown", "main.fpas:1:1"], "Unknown query kind"),
        (vec!["rename", "main.fpas:1:1"], "Missing replacement name"),
        (vec!["type"], "Missing source position"),
        (vec!["type", "main.fpas:0:1"], "Invalid source position"),
        (vec!["type", "main.fpas:1:0"], "Invalid source position"),
        (vec!["type", "main.fpas:1:+2"], "Invalid source position"),
        (
            vec!["type", "main.fpas:1:99999999999999999999999999999"],
            "Invalid source position",
        ),
        (vec!["type", "main.fpas:1"], "Invalid source position"),
        (vec!["type", "main.fpasprj:1:1"], "require a `.fpas`"),
        (
            vec!["type", "main.fpas:1:1", "extra"],
            "exactly one source position",
        ),
        (
            vec!["type", "main.fpas:1:1", "--include-declaration"],
            "Unknown or repeated option",
        ),
        (
            vec![
                "refs",
                "main.fpas:1:1",
                "--include-declaration",
                "--include-declaration",
            ],
            "Unknown or repeated option",
        ),
        (vec!["type", "main.fpas:2:1"], "outside source"),
        (vec!["type", "main.fpas:1:99"], "outside source"),
        (vec!["def", "missing.fpas:1:1"], "Cannot read source"),
        (
            vec!["def", "main.fpas:1:1", "--project", "missing.fpasprj"],
            "manifest",
        ),
    ] {
        let (code, stdout, stderr) = query(&cwd, &args);
        assert_eq!(code, 1, "{args:?}: {stdout} {stderr}");
        assert!(stdout.is_empty(), "{args:?}: {stdout}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
    }
    write_text(
        &cwd.join("main.fpas"),
        "program T; begin discard Missing; end.",
    );
    let (code, stdout, stderr) = query(&cwd, &["type", "main.fpas:1:26"]);
    assert_eq!(code, 1);
    assert!(stdout.is_empty());
    let diagnostic: Value = serde_json::from_str(stderr.trim()).expect("structured query failure");
    assert_eq!(diagnostic["code"], "FP3003");
}

#[test]
fn query_refs_and_def_resolve_fields_of_inferred_record_values() {
    let cwd = create_temp_dir("query-inferred-field");
    write_text(
        &cwd.join("main.fpas"),
        "program T;\ntype Point = record\n  X: integer;\nend record;\nbegin\n  const P := Point(X := 1);\n  discard P.X;\nend.\n",
    );
    let result = report(&cwd, &["refs", "main.fpas:3:3"]);
    let starts = result["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .map(|entry| entry["range"]["start"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        starts,
        [
            json!({"line": 6, "column": 20}),
            json!({"line": 7, "column": 13})
        ]
    );
    let result = report(&cwd, &["def", "main.fpas:7:13"]);
    assert_eq!(
        result["matches"][0]["range"]["start"],
        json!({"line": 3, "column": 3})
    );
}

#[test]
fn query_refs_include_consumers_from_an_explicit_workspace() {
    let cwd = create_temp_dir("query-workspace");
    write_text(
        &cwd.join("suite.fpasworkspace"),
        "[workspace]\nname = \"suite\"\nmembers = [\"lib/lib.fpasprj\", \"app/app.fpasprj\"]\n",
    );
    write_text(
        &cwd.join("lib/lib.fpasprj"),
        "[project]\nname = \"lib\"\nkind = \"library\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("lib/math.fpas"),
        "unit Demo.Math;\npublic const Answer: integer := 42;\nend unit;\n",
    );
    write_text(
        &cwd.join("app/app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("app/main.fpas"),
        "program App;\nuses Demo.Math as M;\nbegin\n  discard M.Answer;\nend.\n",
    );
    let result = report(
        &cwd,
        &[
            "refs",
            "lib/math.fpas:2:14",
            "--project",
            "suite.fpasworkspace",
        ],
    );
    assert_eq!(
        result["matches"].as_array().expect("matches").len(),
        1,
        "{result}"
    );
    assert!(
        result["matches"][0]["path"]
            .as_str()
            .expect("consumer")
            .ends_with("/app/main.fpas")
    );
    assert_eq!(
        result["matches"][0]["range"]["start"],
        json!({"line": 4, "column": 13})
    );
    write_text(
        &cwd.join("other/other.fpasprj"),
        "[project]\nname = \"other\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(&cwd.join("other/main.fpas"), "program Outside; begin end.");
    let (code, stdout, stderr) = query(
        &cwd,
        &[
            "def",
            "other/main.fpas:1:9",
            "--project",
            "suite.fpasworkspace",
        ],
    );
    assert_eq!(code, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("does not belong"), "{stderr}");
}

#[test]
fn query_help_is_available_at_each_level() {
    let cwd = create_temp_dir("query-help");
    for command in [
        vec!["query", "--help"],
        vec!["query", "refs", "--help"],
        vec!["query", "def", "--help"],
        vec!["query", "type", "--help"],
        vec!["query", "rename", "--help"],
    ] {
        let args = command
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect::<Vec<_>>();
        let (code, stdout, stderr) = run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(code, 0, "{stderr}");
        assert!(stderr.is_empty());
        assert!(
            stdout.contains("Examples:") && stdout.contains("fpas query"),
            "{stdout}"
        );
    }
}
