//! `fpas check --fix`: machine-applicable fixes are written in place pass after pass, the
//! remaining diagnostics are reported, and files outside the checked input stay untouched.

use super::*;
use crate::test_support::{write_library_fpasprj, write_program_fpasprj_with_deps};

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

const DELPHI_PROGRAM: &str = "program Demo;\n\nuses Std.Console as Console;\n\nfunction Twice(const X: integer): integer;\nbegin\n  Result := X * 2;\nend function;\n\nprocedure Show;\nbegin\n  var Count: integer := 0;\n  if Count == 0 then\n  begin\n    Inc(Count);\n  end\n  else\n  begin\n    Count := Length(\"abc\");\n  end;\n  { done }\n  Console.WriteLn(Twice(Count));\nend procedure;\n\nbegin\n  Show();\nend.\n";

const FIXED_PROGRAM: &str = "program Demo;\n\nuses Std.Console as Console;\n\nfunction Twice(X: integer): integer;\nbegin\n  return X * 2;\nend function;\n\nprocedure Show();\nbegin\n  var Count: integer := 0;\n  if Count = 0 then\n    Count := Count + 1;\n  else\n    Count := 'abc'.Length();\n  end if;\n  // done\n  Console.WriteLn(Twice(Count));\nend procedure;\n\nbegin\n  Show();\nend.\n";

#[test]
fn check_fix_rewrites_habits_until_the_file_checks() {
    let cwd = create_temp_dir("check-fix-habits");
    let main = cwd.join("main.fpas");
    write_text(&main, DELPHI_PROGRAM);
    let main_arg = main.to_string_lossy().to_string();

    let (exit_code, stdout, stderr) =
        support::run_cli_args_and_capture_output(&args(&["check", "--fix", &main_arg]), &cwd);
    let fixed = fs::read_to_string(&main).expect("fixed source");
    let (second_exit, _, second_stderr) =
        support::run_cli_args_and_capture_output(&args(&["check", "--fix", &main_arg]), &cwd);
    let second = fs::read_to_string(&main).expect("source after second run");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "");
    assert_eq!(stderr, "applied 9 fixes in 1 file\n");
    assert_eq!(fixed, FIXED_PROGRAM);
    assert_eq!(second_exit, 0, "stderr: {second_stderr}");
    assert_eq!(second_stderr, "applied 0 fixes in 0 files\n");
    assert_eq!(second, FIXED_PROGRAM);
}

#[test]
fn check_fix_leaves_review_fixes_and_reports_them_as_json() {
    let cwd = create_temp_dir("check-fix-review");
    let main = cwd.join("main.fpas");
    let source = "program Demo;\n\nbegin\n  const Value: Option of integer := nil;\n  if 1 == 1 then\n    discard Value;\n  end if;\nend.\n";
    write_text(&main, source);

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &args(&[
            "check",
            "--fix",
            "--diagnostics",
            "json",
            &main.to_string_lossy(),
        ]),
        &cwd,
    );
    let fixed = fs::read_to_string(&main).expect("fixed source");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert_eq!(fixed, source.replace("1 == 1", "1 = 1"));
    let records: Vec<serde_json::Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("every stderr line is JSON"))
        .collect();
    assert_eq!(records.len(), 1, "{stderr}");
    assert_eq!(records[0]["fixes"][0]["applicability"], "needs-review");
    assert_eq!(records[0]["fixes"][0]["edits"][0]["new_text"], "None");
}

#[test]
fn check_fix_uses_selected_project_ownership_instead_of_dependency_directory_placement() {
    for (library_path, dependency_path, workspace) in [
        ("libs/base", "../../libs/base/base.fpasprj", false),
        ("apps/demo/libs/base", "libs/base/base.fpasprj", false),
        ("libs/base", "../../libs/base/base.fpasprj", true),
    ] {
        let cwd = create_temp_dir("check-fix-dependencies");
        let library_dir = cwd.join(library_path);
        let app_dir = cwd.join("apps/demo");
        write_library_fpasprj(&library_dir.join("base.fpasprj"), &["src/**/*.fpas"]);
        let library_source = "unit Lib.Base;\npublic function Both(A: boolean; B: boolean): boolean;\nbegin\n  return A && B;\nend function;\nend unit;\n";
        let library_file = library_dir.join("src/base.fpas");
        write_text(&library_file, library_source);
        let app_project = app_dir.join("demo.fpasprj");
        write_program_fpasprj_with_deps(
            &app_project,
            "src/main.fpas",
            &["src/**/*.fpas"],
            &[dependency_path],
        );
        let main = app_dir.join("src/main.fpas");
        write_text(
            &main,
            "program Demo;\nuses Lib.Base as Base;\nbegin\n  if Base.Both(true, false) != true then\n    discard 0;\n  end if;\nend.\n",
        );
        let input = if workspace {
            let manifest = cwd.join("workspace/root.fpasworkspace");
            write_text(
                &manifest,
                "[workspace]\nname = \"root\"\nmembers = [\"../apps/demo/demo.fpasprj\", \"../libs/base/base.fpasprj\"]\n",
            );
            manifest
        } else {
            app_project
        };

        let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
            &args(&["check", "--fix", &input.to_string_lossy()]),
            &app_dir,
        );
        let library_after = fs::read_to_string(&library_file).expect("library source");
        let main_after = fs::read_to_string(&main).expect("main source");
        fs::remove_dir_all(&cwd).expect("temp directory must be removed");

        if workspace {
            assert_eq!(exit_code, 0, "stderr: {stderr}");
            assert_eq!(library_after, library_source.replace("&&", "and"));
        } else {
            assert_eq!(exit_code, 1, "stderr: {stderr}");
            assert_eq!(library_after, library_source);
            assert!(stderr.contains("base.fpas"), "{stderr}");
        }
        assert!(!main_after.contains("!="), "{main_after}");
    }
}

#[test]
fn check_fix_updates_project_owned_sources_outside_the_manifest_directory() {
    for workspace in [false, true] {
        let cwd = create_temp_dir("check-fix-external-source");
        let project_file = cwd.join("project/app.fpasprj");
        write_program_fpasprj_with_deps(&project_file, "../src/main.fpas", &["../src/*.fpas"], &[]);
        let main = cwd.join("src/main.fpas");
        let source = if workspace {
            "program Demo;\nuses Demo.Logic as Logic;\nbegin\n  discard Logic.Both(true, true) == true;\nend.\n"
        } else {
            "program Demo;\nuses Demo.Logic as Logic;\nbegin\n  discard Logic.Both(true, true) = true;\nend.\n"
        };
        write_text(&main, source);
        let unit = cwd.join("src/logic.fpas");
        let unit_source = "unit Demo.Logic;\npublic function Both(A: boolean; B: boolean): boolean;\nbegin\n  return A && B;\nend function;\nend unit;\n";
        write_text(&unit, unit_source);
        let input = if workspace {
            let manifest = cwd.join("workspace/root.fpasworkspace");
            write_text(
                &manifest,
                "[workspace]\nname = \"root\"\nmembers = [\"../project/app.fpasprj\"]\n",
            );
            manifest
        } else {
            project_file
        };

        let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
            &args(&["check", "--fix", &input.to_string_lossy()]),
            &cwd,
        );
        let fixed = fs::read_to_string(&main).expect("main source");
        let fixed_unit = fs::read_to_string(&unit).expect("unit source");
        fs::remove_dir_all(&cwd).expect("temp directory must be removed");

        assert_eq!(
            fixed_unit,
            unit_source.replace("&&", "and"),
            "stderr: {stderr}"
        );
        assert_eq!(fixed, source.replace("==", "="), "stderr: {stderr}");
        assert_eq!(exit_code, 0, "stderr: {stderr}");
    }
}

#[test]
fn fix_is_only_accepted_by_check() {
    let cwd = create_temp_dir("check-fix-option");
    let main = cwd.join("main.fpas");
    write_text(&main, "program Demo;\nbegin\nend.\n");
    let main_arg = main.to_string_lossy().to_string();

    let (run_exit, _, run_stderr) =
        support::run_cli_args_and_capture_output(&args(&["run", "--fix", &main_arg]), &cwd);
    let (duplicate_exit, _, duplicate_stderr) = support::run_cli_args_and_capture_output(
        &args(&["check", "--fix", "--fix", &main_arg]),
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(run_exit, 1, "{run_stderr}");
    assert_eq!(duplicate_exit, 1);
    assert!(
        duplicate_stderr.contains("Duplicate `--fix` option."),
        "{duplicate_stderr}"
    );
}

#[test]
fn check_fix_rewrites_an_ada_main_procedure_and_keeps_its_context_clause_reported() {
    let cwd = create_temp_dir("check-fix-ada-main");
    let main = cwd.join("main.fpas");
    write_text(
        &main,
        "with Ada.Text_IO; use Ada.Text_IO;\n\nprocedure Main is\nbegin\n  null;\nend Main;\n",
    );

    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &args(&["check", "--fix", &main.to_string_lossy()]),
        &cwd,
    );
    let fixed = fs::read_to_string(&main).expect("fixed source");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1, "{stderr}");
    assert_eq!(
        fixed,
        "with Ada.Text_IO; use Ada.Text_IO;\n\nprogram Main;\nbegin\n  null;\nend.\n"
    );
    assert!(
        stderr.contains("FPAS writes `uses Std.Console as Console;`"),
        "{stderr}"
    );
}
