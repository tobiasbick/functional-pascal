//! Contract policy at the real CLI, cache, dependency and executable boundaries.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures need contextual assertions"
)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.temp-data")
            .join(format!(
                "cli-contracts-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).expect("fixture root");
        Self(path.canonicalize().expect("canonical root"))
    }
    fn write(&self, name: &str, source: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("source directory");
        fs::write(path, source).expect("source");
    }
    fn run(&self, args: &[&str], expected: i32) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process");
        assert_eq!(
            output.status.code(),
            Some(expected),
            "{args:?}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn source_policy_skips_both_clauses_and_their_evaluation_but_keeps_type_errors_and_assertions() {
    let fixture = Fixture::new();
    for clause in [
        "requires Value > 0;",
        "requires 10 div Value > 0;",
        "ensures (Returned): Returned < 0;",
        "ensures 10 div Value > 0;",
    ] {
        fixture.write(
            "main.fpas",
            &format!(
                "program Main; uses Std.Console as Console;
            function Guard(Value: integer): integer; {clause} begin return Value + 1; end function;
            begin Console.WriteLn(Guard(0)); end."
            ),
        );
        fixture.run(&["run", "main.fpas"], 2);
        let output = fixture.run(&["run", "main.fpas", "--no-contracts"], 0);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
    }
    let output = fixture.run(&["debug", "main.fpas", "--no-contracts"], 0);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("batch JSON");
    assert_eq!(report["outcome"], "completed");
    for clause in ["requires 1;", "ensures (Returned): MissingName > Returned;"] {
        fixture.write("main.fpas", &format!("program Main; function Guard(Value: integer): integer; {clause} begin return Value; end function; begin discard Guard(0); end."));
        fixture.run(&["run", "main.fpas", "--no-contracts"], 1);
    }
    fixture.write("main.fpas", "program Main; uses Std.Test as Test; procedure Guard(); requires false; begin Test.AssertTrue(false); end procedure; begin Guard(); end.");
    fixture.run(&["run", "main.fpas", "--no-contracts"], 2);
    fixture.run(&["check", "main.fpas", "--no-contracts"], 1);
    fixture.run(&["run", "main.fpas", "--no-contracts", "--no-contracts"], 1);
}

fn project(fixture: &Fixture) {
    fixture.write("bottom/bottom.fpasprj", "[project]\nname='bottom'\nkind='library'\n[sources]\ninclude=['*.fpas']\n[exports]\nunits=['Bottom']\n");
    fixture.write("bottom/bottom.fpas", "unit Bottom; public function Guard(Value: integer): integer; ensures (Returned): Returned < 0; begin return Value + 1; end function; end unit;");
    fixture.write("top/top.fpasprj", "[project]\nname='top'\nkind='library'\n[sources]\ninclude=['*.fpas']\n[exports]\nunits=['Top']\n[dependencies]\nprojects=['../bottom/bottom.fpasprj']\n");
    fixture.write("top/top.fpas", "unit Top; uses Bottom as Base; public function Guard(Value: integer): integer; requires Value > 0; begin return Base.Guard(Value); end function; end unit;");
    fixture.write("app/app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n[dependencies]\nprojects=['../top/top.fpasprj']\n");
    fixture.write("app/main.fpas", "program Main; uses Top as Api, Std.Console as Console; begin const Invoke: function(Value: integer): integer := Api.Guard; Console.WriteLn(Invoke(0)); end.");
}

#[test]
fn policy_switch_rebuilds_transitive_sidecars_and_program_images_without_changing_sources() {
    let fixture = Fixture::new();
    project(&fixture);
    fixture.run(&["run", "app/app.fpasprj"], 2);
    let enabled = fs::read(fixture.0.join("bottom/bottom.fpascu")).expect("enabled sidecar");
    let built = fixture.run(&["build", "app/app.fpasprj", "--no-contracts"], 0);
    let build_output = String::from_utf8(built.stdout).expect("build output");
    let source_root = build_output
        .lines()
        .find_map(|line| line.strip_prefix("Debug source root: "))
        .expect("debug source root");
    let disabled = fs::read(fixture.0.join("bottom/bottom.fpascu")).expect("disabled sidecar");
    assert_ne!(
        enabled, disabled,
        "changing policy must replace the cached unit"
    );
    let artifact = fs::read(fixture.0.join("app/app.fpascp")).expect("program image");
    assert!(
        !fpas_program::decode(&artifact)
            .expect("decoded image")
            .identity()
            .contracts_enabled
    );
    for args in [
        vec!["run", "app/app.fpascp"],
        vec!["run", "app/app.fpascp", "--no-contracts"],
        vec!["run", "app/app.fpasprj", "--no-contracts"],
    ] {
        let output = fixture.run(&args, 0);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
    }
    let warm = fixture.run(&["build", "app/app.fpasprj", "--no-contracts"], 0);
    assert!(String::from_utf8_lossy(&warm.stdout).contains("Reused"));
    assert_eq!(
        disabled,
        fs::read(fixture.0.join("bottom/bottom.fpascu")).expect("reused sidecar")
    );
    let output = fixture.run(
        &[
            "debug",
            "app/app.fpascp",
            "--source-root",
            source_root,
            "--no-contracts",
        ],
        0,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug result");
    assert_eq!(report["outcome"], "completed");
    fixture.run(
        &[
            "build",
            "app/app.fpasprj",
            "--executable",
            "--name",
            "unchecked",
            "--no-contracts",
        ],
        0,
    );
    let native = fixture.0.join(if cfg!(windows) {
        "app/unchecked.exe"
    } else {
        "app/unchecked"
    });
    let output = Command::new(native)
        .current_dir(&fixture.0)
        .output()
        .expect("native program");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
    fixture.run(&["run", "app/app.fpasprj"], 2);
    assert_eq!(
        enabled,
        fs::read(fixture.0.join("bottom/bottom.fpascu")).expect("restored policy")
    );
    for args in [
        vec!["run", "app/app.fpascp", "--no-contracts"],
        vec![
            "debug",
            "app/app.fpascp",
            "--source-root",
            ".",
            "--no-contracts",
        ],
    ] {
        let output = fixture.run(&args, 1);
        let text = String::from_utf8_lossy(&output.stderr).to_string()
            + &String::from_utf8_lossy(&output.stdout);
        assert!(
            text.contains("Rebuild") && text.contains("--no-contracts"),
            "{text}"
        );
    }
    assert_eq!(
        fs::read_to_string(fixture.0.join("top/top.fpas")).expect("source"),
        "unit Top; uses Bottom as Base; public function Guard(Value: integer): integer; requires Value > 0; begin return Base.Guard(Value); end function; end unit;"
    );
}

#[test]
fn test_runner_applies_policy_to_parallel_bodies_hooks_and_isolated_timeout_workers() {
    let fixture = Fixture::new();
    fixture.write(
        "tests.fpasprj",
        "[project]\nname='tests'\nkind='test'\n[sources]\ninclude=['*.fpas']\n",
    );
    fixture.write("Helpers.fpas", "unit Helpers;
        public procedure Setup(); requires false; begin null; end procedure;
        public procedure Teardown(); ensures false; begin null; end procedure;
        public function Guard(Value: integer): integer; requires Value > 0; ensures (Returned): Returned < 0; begin return Value + 1; end function; end unit;");
    for name in ["first_test.fpas", "second_test.fpas"] {
        fixture.write(name, "program CheckPolicy; uses Helpers as Helper, Std.Test as Test; begin Test.AssertEquals(1, Helper.Guard(0)); end.");
    }
    fixture.run(
        &["test", "tests.fpasprj", "--jobs", "2", "--timeout", "5"],
        3,
    );
    fixture.run(
        &[
            "test",
            "tests.fpasprj",
            "--jobs",
            "2",
            "--timeout",
            "5",
            "--no-contracts",
        ],
        0,
    );
    fixture.run(
        &["test", "tests.fpasprj", "--jobs", "1", "--no-contracts"],
        0,
    );
}
