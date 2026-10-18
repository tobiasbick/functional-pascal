//! Rename transport and filesystem behavior through the real CLI entry point.

use super::*;
use std::path::PathBuf;

const UNIT: &str = "unit Demo.Math;\npublic function Answer(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n";
const MAIN: &str = "program App;\r\nuses Demo.Math as M;\r\nbegin\r\n  discard '😀 Answer'; discard M.answer();\r\n  discard M.ANSWER(); // Answer\r\nend.\r\n";
const RENAMED_UNIT: &str = "unit Demo.Math;\npublic function Compute(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n";
const RENAMED_MAIN: &str = "program App;\r\nuses Demo.Math as M;\r\nbegin\r\n  discard '😀 Answer'; discard M.Compute();\r\n  discard M.Compute(); // Answer\r\nend.\r\n";

fn workspace() -> PathBuf {
    let cwd = create_temp_dir("query-rename");
    write_text(
        &cwd.join("suite.fpasworkspace"),
        "[workspace]\nname = \"suite\"\nmembers = [\"lib/lib.fpasprj\", \"app/app.fpasprj\"]\n",
    );
    write_text(
        &cwd.join("lib/lib.fpasprj"),
        "[project]\nname = \"lib\"\nkind = \"library\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("app/app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(&cwd.join("lib/math.fpas"), UNIT);
    write_text(&cwd.join("app/main.fpas"), MAIN);
    cwd
}

#[test]
fn query_rename_previews_and_applies_workspace_edits_preserving_source_bytes() {
    let cwd = workspace();
    let args = [
        "rename",
        "lib/math.fpas:2:17",
        "Compute",
        "--project",
        "suite.fpasworkspace",
    ];
    let mut preview_args = args.to_vec();
    preview_args.push("--dry-run");
    let preview = report(&cwd, &preview_args);
    let main_path = cwd
        .join("app/main.fpas")
        .to_string_lossy()
        .replace('\\', "/");
    let unit_path = cwd
        .join("lib/math.fpas")
        .to_string_lossy()
        .replace('\\', "/");
    assert_eq!(
        preview,
        json!({
            "schemaVersion": 1, "query": "rename",
            "position": {"path": unit_path, "line": 2, "column": 17},
            "new_name": "Compute", "applied": false,
            "edits": [
                {"path": main_path, "range": {"start": {"line": 5, "column": 13}, "end": {"line": 5, "column": 19}}, "old_text": "ANSWER", "new_text": "Compute"},
                {"path": main_path, "range": {"start": {"line": 4, "column": 33}, "end": {"line": 4, "column": 39}}, "old_text": "answer", "new_text": "Compute"},
                {"path": unit_path, "range": {"start": {"line": 2, "column": 17}, "end": {"line": 2, "column": 23}}, "old_text": "Answer", "new_text": "Compute"}
            ]
        })
    );
    assert_eq!(
        fs::read(cwd.join("app/main.fpas")).unwrap(),
        MAIN.as_bytes()
    );
    assert_eq!(
        fs::read(cwd.join("lib/math.fpas")).unwrap(),
        UNIT.as_bytes()
    );
    assert_eq!(fs::read_dir(cwd.join("app")).unwrap().count(), 2);
    assert_eq!(fs::read_dir(cwd.join("lib")).unwrap().count(), 2);

    let applied = report(&cwd, &args);
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["edits"], preview["edits"]);
    assert_eq!(
        fs::read(cwd.join("app/main.fpas")).unwrap(),
        RENAMED_MAIN.as_bytes()
    );
    assert_eq!(
        fs::read(cwd.join("lib/math.fpas")).unwrap(),
        RENAMED_UNIT.as_bytes()
    );
    let repeated = report(&cwd, &args);
    assert_eq!(repeated["applied"], true);
    assert_eq!(repeated["edits"], json!([]));
}

#[test]
fn query_rename_updates_inferred_record_fields_without_renaming_shadowed_locals() {
    let cwd = create_temp_dir("query-rename-field");
    let original = "program T;\ntype Point = record\n  X: integer;\nend record;\nbegin\n  const X := 5;\n  const P := Point(X := X);\n  discard P.X;\nend.\n";
    write_text(&cwd.join("main.fpas"), original);
    let result = report(&cwd, &["rename", "main.fpas:8:13", "Left"]);
    assert_eq!(result["edits"].as_array().unwrap().len(), 3);
    assert_eq!(
        fs::read_to_string(cwd.join("main.fpas")).unwrap(),
        "program T;\ntype Point = record\n  Left: integer;\nend record;\nbegin\n  const X := 5;\n  const P := Point(Left := X);\n  discard P.Left;\nend.\n"
    );
}

#[test]
fn query_rename_rejects_unsafe_requests_without_success_output_or_changes() {
    let cwd = create_temp_dir("query-rename-reject");
    let source = "program T;\nconst Source: integer := 1;\nfunction Read(): integer;\nbegin\n  const Captured := 2;\n  return Source + Captured;\nend function;\nbegin\n  discard Read();\nend.\n";
    write_text(&cwd.join("main.fpas"), source);
    for (args, message) in [
        (
            vec!["rename", "main.fpas:2:7", "begin"],
            "not a valid rename target",
        ),
        (
            vec!["rename", "main.fpas:2:7", "Välue", "--dry-run"],
            "not a valid rename target",
        ),
        (vec!["rename", "main.fpas:2:7", "Captured"], "conflicts"),
        (
            vec!["rename", "main.fpas:1:9", "Other"],
            "Program and unit names",
        ),
        (vec!["rename", "main.fpas:9:3", "Other"], "No renameable"),
        (
            vec!["rename", "main.fpas:2:7", "Other", "--dry-run", "--dry-run"],
            "Unknown or repeated option",
        ),
        (
            vec!["rename", "main.fpas:2:7", "Other", "extra"],
            "exactly one source position",
        ),
        (
            vec!["type", "main.fpas:2:7", "--dry-run"],
            "Unknown or repeated option",
        ),
    ] {
        let (code, stdout, stderr) = query(&cwd, &args);
        assert_eq!(code, 1, "{args:?}: {stderr}");
        assert!(stdout.is_empty(), "{args:?}: {stdout}");
        assert!(stderr.contains(message), "{args:?}: {stderr}");
        assert_eq!(fs::read_to_string(cwd.join("main.fpas")).unwrap(), source);
    }
}

#[test]
fn query_rename_rejects_external_dependencies_and_errors_in_consumers() {
    let cwd = workspace();
    let (code, stdout, stderr) = query(
        &cwd,
        &[
            "rename",
            "app/main.fpas:5:13",
            "Compute",
            "--project",
            "app/app.fpasprj",
        ],
    );
    assert_eq!(code, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("outside"), "{stderr}");
    let broken_main =
        "program App;\nuses Demo.Math as M;\nbegin discard M.Answer() + Missing; end.\n";
    write_text(&cwd.join("app/main.fpas"), broken_main);
    for mode in [vec![], vec!["--dry-run"]] {
        let mut args = vec![
            "rename",
            "lib/math.fpas:2:17",
            "Compute",
            "--project",
            "suite.fpasworkspace",
        ];
        args.extend(mode);
        let (code, stdout, stderr) = query(&cwd, &args);
        assert_eq!(code, 1, "{stderr}");
        assert!(stdout.is_empty());
        assert!(stderr.contains("FP3003"), "{stderr}");
        assert_eq!(fs::read_to_string(cwd.join("lib/math.fpas")).unwrap(), UNIT);
        assert_eq!(
            fs::read_to_string(cwd.join("app/main.fpas")).unwrap(),
            broken_main
        );
    }
}

#[test]
fn query_rename_protects_a_source_standard_library_inside_the_selected_root() {
    let cwd = create_temp_dir("query-rename-stdlib");
    let main = "program App;\nuses Std.Custom as C;\nbegin\n  discard C.Answer;\nend.\n";
    let unit = "unit Std.Custom;\npublic const Answer: integer := 42;\nend unit;\n";
    write_text(
        &cwd.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"main.fpas\"]\n",
    );
    write_text(&cwd.join("main.fpas"), main);
    write_text(&cwd.join("std/custom.fpas"), unit);
    write_text(
        &cwd.join("std/stdlib.fpasprj"),
        "[project]\nname = \"std\"\nkind = \"library\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    let args = [
        "query",
        "rename",
        "main.fpas:4:13",
        "Changed",
        "--std-lib",
        "std",
    ]
    .map(str::to_owned);
    let (code, stdout, stderr) = run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(code, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("outside the standard library"), "{stderr}");
    assert_eq!(fs::read_to_string(cwd.join("main.fpas")).unwrap(), main);
    assert_eq!(
        fs::read_to_string(cwd.join("std/custom.fpas")).unwrap(),
        unit
    );
}

#[test]
fn query_rename_stages_all_files_before_publishing_any_source() {
    let cwd = workspace();
    let unit_path = cwd.join("lib/math.fpas");
    let original_permissions = fs::metadata(&unit_path).unwrap().permissions();
    let mut readonly = original_permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&unit_path, readonly).unwrap();
    let (code, stdout, stderr) = query(
        &cwd,
        &[
            "rename",
            "lib/math.fpas:2:17",
            "Compute",
            "--project",
            "suite.fpasworkspace",
        ],
    );
    fs::set_permissions(&unit_path, original_permissions).unwrap();
    assert_eq!(code, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("read-only"), "{stderr}");
    assert_eq!(fs::read_to_string(cwd.join("app/main.fpas")).unwrap(), MAIN);
    assert_eq!(fs::read_to_string(unit_path).unwrap(), UNIT);
    assert_eq!(
        fs::read_dir(cwd.join("app")).unwrap().count(),
        2,
        "no staged file survives"
    );
}

#[cfg(windows)]
#[test]
fn query_rename_restores_committed_sources_when_a_later_replacement_fails() {
    use std::os::windows::fs::OpenOptionsExt;

    let cwd = workspace();
    // Allow reads and writes but deny replacement of the last file. Staging can finish,
    // and publication of the first file succeeds before this real OS failure occurs.
    let locked = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(cwd.join("lib/math.fpas"))
        .unwrap();
    let (code, stdout, stderr) = query(
        &cwd,
        &[
            "rename",
            "lib/math.fpas:2:17",
            "Compute",
            "--project",
            "suite.fpasworkspace",
        ],
    );
    drop(locked);
    assert_eq!(code, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("Cannot publish rename"), "{stderr}");
    assert_eq!(fs::read_to_string(cwd.join("app/main.fpas")).unwrap(), MAIN);
    assert_eq!(fs::read_to_string(cwd.join("lib/math.fpas")).unwrap(), UNIT);
}
