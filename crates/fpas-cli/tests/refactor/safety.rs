//! Migrated rename guards and output failures through the canonical plan/apply route.

use super::*;

#[test]
fn checked_field_rename_preserves_shadowed_locals_and_constructor_labels() {
    let f = Fixture::new();
    let original = "program T;\ntype Point = record\n  X: integer;\nend record;\nbegin\n  const X := 5;\n  const P := Point(X := X);\n  discard P.X;\nend.\n";
    f.write("app/main.fpas", original);
    let (prepared, _) = f.run(
        &[
            "rename",
            "app/main.fpas:8:13",
            "--name",
            "Left",
            "--context",
            "suite.fpasworkspace",
            "--std-lib",
            "std",
            "--out",
            "field.json",
        ],
        0,
    );
    assert_eq!(
        prepared["result"]["plan"]["edits"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    f.run(&["apply", "--plan", "field.json"], 0);
    assert_eq!(
        fs::read_to_string(f.0.join("app/main.fpas")).unwrap(),
        "program T;\ntype Point = record\n  Left: integer;\nend record;\nbegin\n  const X := 5;\n  const P := Point(Left := X);\n  discard P.Left;\nend.\n"
    );
}

#[test]
fn unsafe_names_capture_and_nonrenameable_targets_remain_explicit_blockers() {
    let f = Fixture::new();
    let source = "program T;\nconst Source: integer := 1;\nfunction Read(): integer;\nbegin\n  const Captured := 2;\n  return Source + Captured;\nend function;\nbegin\n  discard Read();\nend.\n";
    f.write("app/main.fpas", source);
    for (position, name) in [
        ("2:7", "begin"),
        ("2:7", "Välue"),
        ("2:7", "Captured"),
        ("1:9", "Other"),
        ("9:3", "Other"),
    ] {
        let selected = format!("app/main.fpas:{position}");
        let (report, _) = f.run(
            &[
                "rename",
                &selected,
                "--name",
                name,
                "--context",
                "suite.fpasworkspace",
                "--std-lib",
                "std",
            ],
            1,
        );
        assert_eq!(report["result"]["status"], "unsupported", "{report}");
        assert!(report["result"]["plan"].is_null());
        assert!(!report["result"]["blockers"].as_array().unwrap().is_empty());
        assert_eq!(
            fs::read_to_string(f.0.join("app/main.fpas")).unwrap(),
            source
        );
    }
}

#[test]
fn external_sources_standard_library_and_broken_consumers_block_edits() {
    let f = Fixture::new();
    let (external, _) = f.run(
        &[
            "rename",
            "app/main.fpas:5:13",
            "--name",
            "Compute",
            "--context",
            "app/app.fpasprj",
            "--std-lib",
            "std",
        ],
        1,
    );
    assert_eq!(
        external["result"]["blockers"][0]["code"],
        "outside_edit_scope"
    );
    f.write(
        "app/main.fpas",
        "program App; uses Demo.Math as M; begin discard M.Answer() + Missing; end.",
    );
    let (broken, _) = f.preview(&[], 3);
    assert_eq!(broken["result"]["status"], "incomplete");
    assert!(
        broken["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "FP3003")
    );
    assert_eq!(fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(), UNIT);
    f.write(
        "std/custom.fpas",
        "unit Std.Custom; public const Answer: integer := 42; end unit;",
    );
    f.write(
        "app/main.fpas",
        "program App;\nuses Std.Custom as C;\nbegin\n  discard C.Answer;\nend.\n",
    );
    let (standard, _) = f.run(
        &[
            "rename",
            "app/main.fpas:4:13",
            "--name",
            "Changed",
            "--context",
            "suite.fpasworkspace",
            "--std-lib",
            "std",
        ],
        1,
    );
    assert_eq!(
        standard["result"]["blockers"][0]["code"],
        "outside_edit_scope"
    );
    assert!(
        fs::read_to_string(f.0.join("std/custom.fpas"))
            .unwrap()
            .contains("Answer")
    );
}

#[test]
fn final_stdout_failure_reports_that_saved_files_must_be_inspected() {
    let f = Fixture::new();
    f.preview(&["--out", "rename.json"], 0);
    let mut child = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .args(["refactor", "apply", "--plan", "rename.json"])
        .current_dir(&f.0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("Inspect saved files before retrying")
    );
    assert_eq!(
        fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(),
        UNIT.replace("Answer", "Compute")
    );
    assert_eq!(
        f.run(&["apply", "--plan", "rename.json"], 0).0["result"]["status"],
        "already_applied"
    );
}

#[test]
fn invalid_requests_and_oversized_plans_use_bounded_schema_two_errors() {
    let f = Fixture::new();
    for args in [
        vec![
            "rename",
            "app/main.fpas:4:33",
            "--name",
            "Compute",
            "--dry-run",
        ],
        vec![
            "rename",
            "app/main.fpas:4:33",
            "--symbol",
            "Answer",
            "--name",
            "Compute",
        ],
        vec!["rename", "app/main.fpas:4:33", "Compute"],
        vec!["extract-variable", "--range", "app/main.fpas:4:33:4:12"],
        vec![
            "apply",
            "--plan",
            "missing.json",
            "--context",
            "suite.fpasworkspace",
        ],
    ] {
        let (report, _) = f.run(&args, 1);
        assert_eq!(report["schema_version"], 2);
        assert_eq!(report["diagnostics"][0]["code"], "invalid_request");
    }
    fs::write(f.0.join("huge.json"), vec![b' '; 8 * 1024 * 1024 + 1]).unwrap();
    let (oversized, bytes) = f.run(&["apply", "--plan", "huge.json"], 1);
    assert_eq!(oversized["diagnostics"][0]["code"], "plan_read");
    assert!(bytes.len() < 4096);
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .args(["query", "rename", "app/main.fpas:4:33", "Compute"])
        .current_dir(&f.0)
        .output()
        .unwrap();
    let (removed, _) = decode(output, 1);
    assert_eq!(removed["diagnostics"][0]["code"], "invalid_request");
    assert!(removed["operation"].is_null());
}

#[test]
fn focused_refactoring_help_has_examples_and_does_not_start_preparation() {
    let f = Fixture::new();
    for command in [
        "rename",
        "add-parameter",
        "extract-variable",
        "organize-imports",
        "apply",
    ] {
        let output = f.output(&[command, "--help"], &f.0);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains(&format!("fpas refactor {command}")));
        assert!(help.contains("Example:"));
        assert!(!help.contains("--dry-run"));
    }
}
