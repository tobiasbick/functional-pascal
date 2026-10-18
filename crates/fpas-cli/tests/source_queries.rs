//! Source-query JSON, saved inputs and process boundaries through the installed CLI.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Fixtures fail at their observable protocol boundary."
)]

#[path = "source_queries/mod.rs"]
mod cases;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-source-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let f = Self(root);
        f.write("main.fpas", source);
        f
    }
    fn write(&self, name: &str, source: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command.current_dir(&self.0);
        command
    }
    fn run(&self, args: &[&str], exit: i32) -> Value {
        let mut command = self.command();
        command.arg("query").args(args);
        if !args.contains(&"--std-lib") {
            command.args([
                "--std-lib",
                concat!(env!("CARGO_MANIFEST_DIR"), "/../../lib"),
            ]);
        }
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let prefix = format!("fpas-query-{}-", child.id());
        let output = child.wait_with_output().unwrap();
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(output.stdout.ends_with(b"\n"));
        assert!(output.stdout.len() <= 8 * 1024 * 1024);
        let report: Value = serde_json::from_slice(&output.stdout).expect("one JSON object");
        assert_eq!(report["schema_version"], 1);
        assert_eq!(report["exit_code"], exit);
        for key in [
            "operation",
            "outcome",
            "request",
            "context",
            "target",
            "matches",
            "type",
            "diagnostics",
            "coverage",
            "limits",
        ] {
            assert!(report.get(key).is_some(), "{key}");
        }
        assert!(
            !fs::read_dir(std::env::temp_dir())
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(&prefix)),
            "worker scratch leaked"
        );
        report
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const LOCAL: &str =
    "program T;\nbegin\n  const Count := 1;\n  discard Count;\n  discard COUNT;\nend.\n";

#[test]
fn references_separate_identity_from_empty_missing_degraded_and_limited_results() {
    let f = Fixture::new(LOCAL);
    let r = f.run(&["refs", "main.fpas:3:9"], 0);
    assert_eq!(r["operation"], "refs");
    assert_eq!(r["outcome"], "ok");
    assert_eq!(r["target"]["name"], "Count");
    assert_eq!(r["target"]["declaration"]["path"], "main.fpas");
    assert_eq!(r["matches"].as_array().unwrap().len(), 2);
    assert_eq!(
        r["matches"][0]["range"]["start"],
        json!({"line":4,"column":11})
    );
    assert_eq!(r["coverage"]["complete"], true);
    assert!(
        r["coverage"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["path"] == "main.fpas"
                && file["sha256"]
                    == "e64c7e89481f78b0a27a070d884bd20a6b97ce3f61bd02fabbce6284bacf2c6e")
    );
    let limited = f.run(&["refs", "main.fpas:3:9", "--limit", "1"], 3);
    assert_eq!(limited["matches"].as_array().unwrap().len(), 1);
    assert_eq!(limited["coverage"]["reasons"], json!(["result_limit"]));
    assert!(limited["coverage"]["total_count"].is_null());
    f.write("main.fpas", "program T; begin const Unused := 1; end.");
    let unused = f.run(&["refs", "main.fpas:1:24"], 0);
    assert_eq!(unused["matches"], json!([]));
    assert_eq!(unused["target"]["name"], "Unused");
    assert_eq!(unused["coverage"]["total_count"], 0);
    f.write(
        "main.fpas",
        "program T; begin const Count := 1; discard Count; discard Missing; end.",
    );
    let broken = f.run(&["refs", "main.fpas:1:24"], 3);
    assert_eq!(broken["matches"].as_array().unwrap().len(), 1);
    assert!(
        broken["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "FP3003" && d["source"] == "main.fpas")
    );
}

#[test]
fn strict_requests_and_token_boundaries_use_stdout_json() {
    let f = Fixture::new(LOCAL);
    for args in [
        vec![],
        vec!["def", "main.fpas:3:9"],
        vec!["refs"],
        vec!["refs", "main.fpas:0:1"],
        vec!["refs", "main.fpas:3:999"],
        vec!["refs", "absent.fpas:1:1"],
        vec!["refs", "main.fpas:3:9", "--include-declaration"],
        vec!["refs", "main.fpas:3:9", "--context", "."],
        vec!["refs", "main.fpas:3:9", "--project", "app.fpasprj"],
        vec!["type", "main.fpas:3:9", "--dry-run"],
        vec!["type", "main.fpas:3:9", "--limit", "1"],
        vec!["refs", "main.fpas:3:9", "--limit", "10001"],
        vec!["refs", "main.fpas:3:9", "--timeout", "0"],
        vec!["refs", "main.fpas:3:9", "--timeout", "1", "--timeout", "2"],
    ] {
        assert_eq!(f.run(&args, 1)["outcome"], "error", "{args:?}");
    }
    for position in ["main.fpas:3:14", "main.fpas:3:2", "main.fpas:7:1"] {
        let r = f.run(&["refs", position], 1);
        assert_eq!(r["outcome"], "not_found");
        assert_eq!(r["coverage"]["complete"], true);
    }
}

#[test]
fn references_include_workspace_consumers_and_separate_test_programs() {
    let f = Fixture::new("program T; begin end.");
    f.write(
        "lib/lib.fpasprj",
        "[project]\nname='core'\nkind='library'\n[sources]\ninclude=['*.fpas']\n",
    );
    f.write(
        "lib/core.fpas",
        "unit Demo.Core; public const Answer: integer := 42; end unit;",
    );
    f.write("tests/tests.fpasprj","[project]\nname='tests'\nkind='test'\n[sources]\ninclude=['*.fpas']\n[dependencies]\nprojects=['../lib/lib.fpasprj']\n");
    for name in ["a_test.fpas", "b_test.fpas"] {
        f.write(
            &format!("tests/{name}"),
            "program Test; uses Demo.Core as C; begin discard C.Answer; end.",
        );
    }
    f.write(
        "suite.fpasworkspace",
        "[workspace]\nname='suite'\nmembers=['lib/lib.fpasprj','tests/tests.fpasprj']\n",
    );
    let r = f.run(
        &[
            "refs",
            "lib/core.fpas:1:30",
            "--context",
            "suite.fpasworkspace",
        ],
        0,
    );
    assert_eq!(r["matches"].as_array().unwrap().len(), 2, "{r}");
    assert_eq!(r["matches"][0]["path"], "tests/a_test.fpas");
    assert_eq!(r["matches"][1]["path"], "tests/b_test.fpas");
    assert_eq!(r["context"]["kind"], "workspace");
    assert_eq!(
        f.run(
            &["refs", "main.fpas:1:9", "--context", "suite.fpasworkspace"],
            1
        )["diagnostics"][0]["code"],
        "outside_context"
    );
    assert!(!f.0.join("main.fpascp").exists());
}

#[test]
fn input_byte_and_file_bounds_do_not_become_empty_successes() {
    let f = Fixture::new(&" ".repeat(8 * 1024 * 1024 + 1));
    assert!(
        f.run(&["refs", "main.fpas:1:1"], 3)["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("source_limit"))
    );
    f.write("main.fpas", LOCAL);
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='library'\n[sources]\ninclude=['units/*.fpas']\n",
    );
    for index in 0..4097 {
        f.write(
            &format!("units/u{index}.fpas"),
            &format!("unit Demo.U{index}; end unit;"),
        );
    }
    assert!(
        f.run(
            &["refs", "units/u0.fpas:1:6", "--context", "app.fpasprj"],
            3
        )["coverage"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("source_limit"))
    );
}
