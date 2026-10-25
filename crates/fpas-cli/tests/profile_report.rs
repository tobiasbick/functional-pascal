//! Real CLI boundaries for `fpas run --profile` reports.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures assert filesystem and JSON behavior"
)]

use serde_json::Value;
#[path = "profile_report/debug.rs"]
mod debug;
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
    assert_eq!(report["version"], 2);
    assert_eq!(report["status"], "ok");
    assert!(report["instructions"].as_u64().expect("instructions") > 300_000);
    let parallelism = report["parallelism"].as_f64().expect("parallelism");
    assert!(parallelism > 0.0 && parallelism <= 1.5, "{parallelism}");
    let routines = &report["routines"];
    assert!(routines["total"].as_u64().expect("routine total") >= 2);
    let hot = &routines["rows"][0];
    assert_eq!(hot["name"], "hot");
    assert_eq!(hot["source"], "main.fpas");
    assert!(hot["self_time_percent"].as_f64().expect("time share") > 50.0);
    assert!(hot["self_instructions"].as_u64().expect("instructions") > 0);
    assert_eq!(report["lines"]["rows"][0]["routine"], "hot");
    assert!(report["lines"]["rows"][0]["time_percent"].is_f64());
    assert!(report["counters"]["collection_passes"].is_u64());
    let findings = report["findings"].as_array().expect("findings");
    let dominant = findings
        .iter()
        .find(|finding| finding["code"] == "dominant_routine")
        .unwrap_or_else(|| panic!("{findings:?}"));
    assert_eq!(dominant["location"]["routine"], "hot");
    assert_eq!(dominant["evidence"][0]["unit"], "percent");
    assert_eq!(dominant["evidence"][0]["denominator"], "sampled_ms");
    let run = &report["run"];
    assert_eq!(run["tool"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(run["target"]["kind"], "source");
    assert_eq!(run["target"]["name"], "main.fpas");
    assert_eq!(run["runtime"]["workers"], 0);
    assert_eq!(run["runtime"]["collection"]["poll_interval"], 256);
    assert_eq!(run["sampling"]["max_batch_instructions"], 256);
    assert_eq!(run["sampling"]["row_limit"], 25);
    assert_eq!(run["sampling"]["counters_exclusive"], true);
}

#[test]
fn run_metadata_identifies_the_executable_without_arguments_or_environment() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", HOT);
    let profile = |report: &str, extra: &[&str]| {
        let mut args = vec!["run", "--profile", report];
        args.extend(extra);
        args.extend(["main.fpas", "--", "SENTINEL_ARGUMENT_7319"]);
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&fixture.0)
            .env("FPAS_PROFILE_SENTINEL", "SENTINEL_ENVIRONMENT_4471")
            .args(&args)
            .output()
            .expect("CLI process");
        exit(&output, 0);
        let bytes = fs::read(fixture.0.join(report)).expect("report");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains("SENTINEL_"),
            "sentinel leaked into the report"
        );
        assert!(!text.contains(fixture.0.to_str().expect("fixture path")));
        fixture.report(report)
    };
    let first = profile("first.json", &[]);
    let second = profile("second.json", &["--gc-poll-interval=77"]);
    let digest = first["run"]["target"]["digest"].as_str().expect("digest");
    assert!(digest.starts_with("blake3:"));
    assert_eq!(second["run"]["target"]["digest"], digest);
    assert_eq!(second["run"]["runtime"]["collection"]["poll_interval"], 77);
    fixture.write("main.fpas", &HOT.replace("300000", "300001"));
    let changed = profile("changed.json", &[]);
    assert_ne!(changed["run"]["target"]["digest"], digest);
}

#[test]
fn sources_outside_the_current_directory_report_only_file_names() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", HOT);
    fs::create_dir_all(fixture.0.join("work")).expect("work directory");
    let program = fixture.0.join("main.fpas");
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(fixture.0.join("work"))
        .args(["run", "--profile", "outside.json"])
        .arg(&program)
        .output()
        .expect("CLI process");
    exit(&output, 0);
    let report = fixture.report("work/outside.json");
    assert_eq!(report["run"]["target"]["name"], "main.fpas");
    let mut sources = Vec::new();
    for table in ["routines", "lines"] {
        for row in report[table]["rows"].as_array().expect("rows") {
            if let Some(source) = row["source"].as_str() {
                sources.push(source.to_owned());
            }
        }
    }
    assert!(
        sources.iter().any(|source| source == "main.fpas"),
        "{sources:?}"
    );
    for source in &sources {
        assert!(!Path::new(source).is_absolute(), "absolute source {source}");
        assert!(!source.contains(':'), "absolute source {source}");
    }
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
