//! Real CLI boundaries for `fpas run --profile` reports.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures assert filesystem and JSON behavior"
)]

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

const HOT: &str = "program Main;
uses Std.Console as Console, Std.Conv as Conv;

function Hot(Limit: integer): integer;
begin
  var Total: integer := 0;
  for Index: integer := 1 to Limit do
    begin
      Total := Total + Index mod 7;
    end;
  end for;
  return Total;
end function;

begin
  Console.WriteLn('total ' + Conv.IntToStr(Hot(300000)));
end.
";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(".temp-data")
            .join(format!(
                "cli-profile-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).expect("fixture directory");
        Self(path.canonicalize().expect("fixture root"))
    }

    fn write(&self, path: &str, text: &str) {
        fs::write(self.0.join(path), text).expect("fixture source");
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process")
    }

    fn report(&self, path: &str) -> Value {
        let bytes = fs::read(self.0.join(path)).expect("published report");
        assert!(bytes.ends_with(b"\n"));
        serde_json::from_slice(&bytes).expect("one complete JSON document")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn exit(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn profile_reports_hot_routine_and_keeps_program_output() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", HOT);
    let plain = fixture.run(&["run", "main.fpas"]);
    exit(&plain, 0);
    let profiled = fixture.run(&["run", "--profile", "out/profile.json", "main.fpas"]);
    exit(&profiled, 0);
    assert_eq!(profiled.stdout, plain.stdout);
    let report = fixture.report("out/profile.json");
    assert_eq!(report["format"], "fpas-profile");
    assert_eq!(report["version"], 1);
    assert_eq!(report["status"], "ok");
    assert!(report["instructions"].as_u64().expect("instructions") > 300_000);
    let hot = &report["routines"][0];
    assert_eq!(hot["name"], "hot");
    assert_eq!(hot["source"], "main.fpas");
    assert!(hot["self_percent"].as_f64().expect("percent") > 90.0);
    assert_eq!(report["lines"][0]["routine"], "hot");
    assert!(report["counters"]["collection_passes"].is_u64());
    let findings = report["findings"].as_array().expect("findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding.as_str().is_some_and(|text| text.contains("`hot`"))),
        "{findings:?}"
    );
}

#[test]
fn aggregate_stores_validate_only_the_stored_value() {
    let fixture = Fixture::new();
    fixture.write(
        "handlers.fpas",
        "program Main;
type Handler = record
  Id: integer;
  Run: function(Value: integer): integer;
end record;

begin
  var Handlers: array of Handler := [];
  for Index: integer := 1 to 2000 do
    begin
      const Offset: integer := Index;
      Handlers.PushInPlace(Handler(Id := Index, Run := function(Value: integer): integer begin
        return Value + Offset;
      end function));
      Handlers[Index - 1] := Handlers[Index - 1];
    end;
  end for;
end.
",
    );
    let output = fixture.run(&["run", "--profile", "handlers.json", "handlers.fpas"]);
    exit(&output, 0);
    let report = fixture.report("handlers.json");
    let visited = report["counters"]["classify_visited"]
        .as_u64()
        .expect("classification visits");
    // Revalidating the growing array on each store would visit about four million values.
    assert!(visited < 100_000, "{visited} classification visits");
}

#[test]
fn profile_is_published_after_failure_and_rejects_existing_destinations() {
    let fixture = Fixture::new();
    fixture.write("fail.fpas", "program Main; begin panic('boom'); end.");
    let failed = fixture.run(&["run", "--profile", "failed.json", "fail.fpas"]);
    exit(&failed, 2);
    let report = fixture.report("failed.json");
    assert_eq!(report["status"], "error");
    assert_eq!(report["exit_code"], 2);
    let existing = fixture.run(&["run", "--profile", "failed.json", "fail.fpas"]);
    exit(&existing, 1);
    assert!(String::from_utf8_lossy(&existing.stderr).contains("already exists"));
    for value in ["-", ""] {
        let rejected = fixture.run(&["run", "--profile", value, "fail.fpas"]);
        exit(&rejected, 1);
    }
    let check = fixture.run(&["check", "--profile", "x.json", "fail.fpas"]);
    assert_ne!(check.status.code(), Some(0));
    assert!(!fixture.0.join("x.json").exists());
}
