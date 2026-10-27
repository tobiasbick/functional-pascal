//! Real debugger processes own the samples, including asynchronous target workers.

use super::*;
use serde_json::json;

#[path = "debug/dap.rs"]
mod dap;

const SOURCE: &str = "program DebugProfileProbe;
uses Std.Console as Console, Std.Tasks as Tasks, Std.Time as Time;

function Hot(Limit: integer): integer;
begin
  var Total := 0;
  for Index: integer := 1 to Limit do
    Total := Total + Index mod 7;
  end for;
  return Total;
end function;

function Background(): integer;
begin
  Time.Sleep(5);
  return Hot(50);
end function;

procedure Marker();
begin
  Console.WriteLn('done');
end procedure;

var Total: integer := 0;
begin
  scope
    const Work := go Background();
    Total := Hot(80);
    Total := Total + Tasks.Wait(Work);
  end scope;
  Marker();
end.
";

fn assert_target(report: &Value, protocol: &str) {
    assert_eq!(report["format"], "fpas-profile");
    assert_eq!(report["version"], 2);
    assert_eq!(report["debug"]["protocol"], protocol);
    assert_eq!(report["debug"]["timing"], "inclusive_wall");
    assert_eq!(report["run"]["runtime"]["execution"], "debug_serial");
    assert_eq!(report["run"]["runtime"]["workers"], 0);
    assert!(
        report["run"]["target"]["digest"]
            .as_str()
            .expect("digest")
            .starts_with("blake3:")
    );
    assert!(report["instructions"].as_u64().expect("samples") > 100);
    let routines = report["routines"]["rows"].as_array().expect("routine rows");
    for name in ["hot", "background"] {
        assert!(
            routines.iter().any(
                |row| row["name"] == name && row["self_instructions"].as_u64().unwrap_or(0) > 0
            ),
            "{routines:?}"
        );
    }
    assert!(
        report["lines"]["rows"]
            .as_array()
            .expect("line rows")
            .iter()
            .any(|row| row["routine"] == "hot" && row["source"] == "main.fpas")
    );
    assert_eq!(report["counters"]["task_spawns"], 1);
    for phase in [
        "resume_ms",
        "schedule_ms",
        "dispatch_ms",
        "root_publication_ms",
    ] {
        assert!(
            report["debug"][phase].as_f64().expect("milliseconds") > 0.0,
            "{phase}: {report}"
        );
    }
    for phase in ["inspection_ms", "wait_ms"] {
        assert!(report["debug"][phase].as_f64().expect("milliseconds") >= 0.0);
    }
    assert!(
        report["routines"]["rows"]
            .as_array()
            .expect("routine rows")
            .len()
            <= 25
    );
    assert!(report["lines"]["rows"].as_array().expect("line rows").len() <= 25);
}

#[test]
fn batch_target_samples_keep_program_output_and_include_spawned_work() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", SOURCE);
    let plain = fixture.run(&["debug", "main.fpas", "--print", "Total"]);
    exit(&plain, 0);
    let profiled = fixture.run(&[
        "debug",
        "main.fpas",
        "--print",
        "Total",
        "--profile",
        "debug.json",
        "--",
        "SENTINEL_DEBUG_ARG_7319",
    ]);
    exit(&profiled, 0);
    let plain: Value = serde_json::from_slice(&plain.stdout).expect("batch result");
    let actual: Value = serde_json::from_slice(&profiled.stdout).expect("batch result");
    for field in ["outcome", "globals", "expressions"] {
        assert_eq!(actual[field], plain[field], "{field}");
    }
    // Captured writes may be drained separately or combined in one output chunk.
    for result in [&plain, &actual] {
        let chunks = result["output"].as_array().expect("output chunks");
        let mut text = String::new();
        for chunk in chunks {
            assert_eq!(chunk["stream"], "stdout");
            assert_eq!(chunk["truncated"], false);
            text.push_str(chunk["text"].as_str().expect("output text"));
        }
        assert_eq!(text, "done\n");
    }
    let report = fixture.report("debug.json");
    assert_target(&report, "batch");
    assert_eq!(report["debug"]["outcome"], "completed");
    assert_eq!(report["status"], "ok");
    let text = serde_json::to_string(&report).expect("serializable report");
    assert!(!text.contains("SENTINEL_DEBUG_ARG_7319"));
    assert!(!text.contains(&fixture.0.to_string_lossy().replace('\\', "/")));
}

#[test]
fn stopped_failed_and_limited_batch_targets_publish_their_actual_outcomes() {
    let fixture = Fixture::new();
    let line = SOURCE
        .lines()
        .position(|line| line == "  Marker();")
        .expect("marker call in fixture")
        + 1;
    let breakpoint = format!("main.fpas:{line}");
    for (source, extra, expected, outcome) in [
        (
            SOURCE.to_owned(),
            vec!["--break", breakpoint.as_str()],
            0,
            "stopped",
        ),
        (
            SOURCE.replace("  Marker();", "  panic('intentional');"),
            vec![],
            2,
            "runtime_error",
        ),
        (
            SOURCE.to_owned(),
            vec!["--instruction-limit", "200"],
            3,
            "error",
        ),
    ] {
        fixture.write("main.fpas", &source);
        let path = format!("{outcome}.json");
        let mut args = vec!["debug", "main.fpas", "--profile", &path];
        args.extend(extra);
        let output = fixture.run(&args);
        exit(&output, expected);
        let report = fixture.report(&path);
        assert_eq!(report["debug"]["outcome"], outcome);
        assert_eq!(report["exit_code"], expected);
        assert!(report["instructions"].as_u64().expect("samples") > 0);
    }
}

#[test]
fn jsonl_profiles_target_execution_and_distinguishes_runtime_from_transport_failure() {
    let fixture = Fixture::new();
    for (outcome, source, expected) in [
        ("completed", SOURCE.to_owned(), 0),
        (
            "runtime_error",
            SOURCE.replace("  Marker();", "  panic('intentional');"),
            2,
        ),
    ] {
        fixture.write("main.fpas", &source);
        let script = [
            json!({"type":"request", "id":1, "command":"initialize", "arguments":{}}),
            json!({"type":"request", "id":2, "command":"launch", "arguments":{"stop_on_entry":false}}),
        ].map(|request| request.to_string() + "\n").concat();
        fixture.write("requests.jsonl", &script);
        let path = format!("{outcome}.json");
        let output = fixture.run(&[
            "debug",
            "main.fpas",
            "--protocol",
            "jsonl",
            "--commands",
            "requests.jsonl",
            "--profile",
            &path,
        ]);
        exit(&output, 0);
        for line in String::from_utf8(output.stdout)
            .expect("UTF-8 protocol")
            .lines()
        {
            serde_json::from_str::<Value>(line).expect("protocol stdout stays JSONL");
        }
        let report = fixture.report(&path);
        assert_target(&report, "jsonl");
        assert_eq!(report["debug"]["outcome"], outcome);
        assert_eq!(report["exit_code"], expected);
    }
}

#[test]
fn debugger_rejects_invalid_profile_destinations_before_executing_target_code() {
    let fixture = Fixture::new();
    fixture.write(
        "main.fpas",
        "program Main; uses Std.Fs as Fs; begin Fs.WriteText('ran.txt', 'ran').Unwrap(); end.",
    );
    fixture.write("existing.json", "keep");
    for protocol in [
        vec![],
        vec!["--protocol", "dap"],
        vec!["--protocol", "jsonl"],
    ] {
        for destination in ["existing.json", "-", ""] {
            let mut args = vec!["debug", "main.fpas", "--profile", destination];
            args.extend(&protocol);
            exit(&fixture.run(&args), 1);
            assert!(!fixture.0.join("ran.txt").exists());
            assert_eq!(
                fs::read_to_string(fixture.0.join("existing.json")).expect("preserved report"),
                "keep"
            );
        }
    }
}
