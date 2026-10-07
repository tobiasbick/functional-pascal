//! Test-worker program stderr records and outcome parity.

use super::*;

#[test]
fn oversized_program_stderr_keeps_test_outcomes_and_reports_in_both_modes() {
    let root = temp_dir();
    let line = format!("{}\n", "x".repeat(4095));
    fs::write(root.join("stderr.txt"), line.repeat(2304)).expect("large child stderr fixture");
    let (command, arguments) = if cfg!(windows) {
        ("cmd", "['/C', 'type stderr.txt>&2']")
    } else {
        ("sh", "['-c', 'cat stderr.txt >&2']")
    };
    for (ending, exit, runtime_code) in [
        ("AssertTrue(true)", 0, None),
        ("AssertTrue(false)", 1, Some("FP5023")),
        ("panic('boom')", 3, Some("FP5010")),
    ] {
        let source = format!(
            "program P; uses Std.Proc, Std.Test; begin const Status: Result of integer, string := Run('{command}', {arguments}); {ending}; end."
        );
        for name in ["first_test.fpas", "second_test.fpas"] {
            write(&root.join(name), &source);
        }
        for jobs in ["1", "2"] {
            let run = |json| {
                let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
                command.current_dir(&root).args([
                    "test",
                    "--report",
                    "json",
                    "--timeout",
                    "30",
                    "--jobs",
                    jobs,
                ]);
                if json {
                    command.args(["--diagnostics", "json"]);
                }
                command.arg(".").output().expect("test CLI starts")
            };
            let json = run(true);
            let text = run(false);
            assert_eq!(json.status.code(), Some(exit), "{ending}, jobs={jobs}");
            assert_eq!(text.status.code(), json.status.code());
            assert_eq!(text.stdout, json.stdout, "test report is unchanged");
            let records = stderr_records(&json);
            assert_eq!(
                records
                    .iter()
                    .filter(|record| record["text"]
                        .as_str()
                        .is_some_and(|text| text.contains("truncated")))
                    .count(),
                2,
                "one truncation event per test"
            );
            let diagnostics = records
                .iter()
                .filter(|record| record["kind"] == "diagnostic")
                .collect::<Vec<_>>();
            if let Some(code) = runtime_code {
                assert_eq!(diagnostics.len(), 2);
                assert!(diagnostics.iter().all(|record| record["code"] == code));
            } else {
                assert!(diagnostics.is_empty());
            }
        }
    }
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn test_workers_forward_child_stderr_as_json_program_output() {
    let root = temp_dir();
    for name in ["first_test.fpas", "second_test.fpas"] {
        write(&root.join(name), &program_source());
    }
    for jobs in ["1", "2"] {
        let run = |json| {
            let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
            command.current_dir(&root).args([
                "test",
                "--report",
                "json",
                "--timeout",
                "10",
                "--jobs",
                jobs,
            ]);
            if json {
                command.args(["--diagnostics", "json"]);
            }
            command.arg(".").output().expect("test CLI starts")
        };
        let json = run(true);
        let text = run(false);
        assert_eq!(json.status.code(), Some(3), "{json:?}");
        assert_eq!(text.status.code(), json.status.code());
        for output in [&json, &text] {
            let report: Value = serde_json::from_slice(&output.stdout).expect("test report JSON");
            assert_eq!(report["summary"]["total"], 2);
            assert_eq!(report["summary"]["runtime_errors"], 2);
        }
        let records = stderr_records(&json);
        assert_eq!(records.len(), 4, "jobs={jobs}: {records:?}");
        for pair in records.chunks_exact(2) {
            assert_eq!(pair[0]["kind"], "program-output");
            assert_eq!(pair[0]["stream"], "stderr");
            assert_eq!(pair[0]["text"], "child err");
            assert_eq!(pair[1]["code"], "FP5010");
        }
    }
    fs::remove_dir_all(root).expect("remove fixtures");
}
