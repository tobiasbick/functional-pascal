//! Exact nested test selection and identity in sequential/parallel JSON reports.

use super::*;
use crate::cli_input::{ResolvedCli, resolve_cli_config};

#[test]
fn exact_test_selection_preserves_nested_identity_and_project_linking() {
    let cwd = create_temp_dir("exact-test-selection");
    write_text(
        &cwd.join("suite.fpasprj"),
        "[project]\nname = 'nested-tests'\nkind = 'test'\n[sources]\ninclude = ['**/*.fpas']\n",
    );
    write_text(
        &cwd.join("support.fpas"),
        "unit Shared; public function Value(): integer; begin return 42 end;",
    );
    for file in [
        "a/same_test.fpas",
        "b/same_test.fpas",
        "a/other_same_test.fpas",
    ] {
        write_text(
            &cwd.join(file),
            "program Nested; uses Shared, Std.Test; begin AssertEquals(42, Value()) end.",
        );
    }
    for jobs in ["1", "2"] {
        let args = [
            "test",
            "--report",
            "json",
            "--jobs",
            jobs,
            "--file",
            "a/same_test.fpas",
            "--file",
            "b/same_test.fpas",
            "suite.fpasprj",
        ]
        .map(str::to_owned);
        let ResolvedCli::Test(config) = resolve_cli_config(&args, &cwd).expect("test arguments")
        else {
            panic!("test command");
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            test_cli(config, &mut stdout, &mut stderr),
            0,
            "{}",
            String::from_utf8_lossy(&stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&stdout).expect("JSON report");
        let tests = report["tests"].as_array().expect("test results");
        assert_eq!(tests.len(), 2);
        for (test, file) in tests.iter().zip(["a/same_test.fpas", "b/same_test.fpas"]) {
            assert_eq!(
                std::path::Path::new(test["file"].as_str().expect("source path")),
                cwd.join(file)
            );
            assert_eq!(test["status"], "pass");
        }
    }
    let args = ["test", "--file", "support.fpas", "suite.fpasprj"].map(str::to_owned);
    let ResolvedCli::Test(config) = resolve_cli_config(&args, &cwd).expect("test arguments") else {
        panic!("test command");
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(test_cli(config, &mut stdout, &mut stderr), 2);
    assert!(stdout.is_empty());
    assert!(String::from_utf8_lossy(&stderr).contains("not in the discovered test set"));
    std::fs::remove_dir_all(cwd).expect("remove fixture");
}
