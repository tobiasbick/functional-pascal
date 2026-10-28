//! Real-process closure admission, portable provenance and bounded active histories.

use super::*;
use serde_json::json;

fn values(report: &Value, watch: usize) -> Vec<Value> {
    report["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["watch_id"] == watch)
        .map(|e| e["new_value"]["scalar"].clone())
        .collect()
}

#[test]
fn source_project_and_image_preserve_nominal_capture_types_and_source_ids() {
    let f = Fixture::new(
        "program T; type Dollars = distinct integer; function Make(Base: Dollars): function(): integer; function Next(): integer; begin return integer(Base); end function; begin return Next; end function; begin const A := Make(Dollars(7)); const B := Make(Dollars(9)); discard A(); discard B(); discard A(); end.",
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let selectors = [
        "--trace",
        "Make.Next::Base",
        "--trace",
        "Make::Base",
        "--trace",
        "make.next::base",
    ];
    let mut args = vec!["debug", "app.fpasprj"];
    args.extend(selectors);
    let r = f.run(&args, 0);
    assert_eq!(values(&r, 0), vec![json!("7"), json!("9"), json!("7")]);
    assert_eq!(r["trace"]["watches"].as_array().unwrap().len(), 2);
    let capture = &r["trace"]["watches"][0];
    assert_eq!(capture["type"], "Dollars");
    assert_eq!(capture["identity"]["kind"], "capture");
    assert_eq!(
        capture["identity"]["capture_source"]["function_id"],
        r["trace"]["watches"][1]["identity"]["function_id"]
    );
    assert_eq!(
        capture["identity"]["capture_source"]["binding_id"],
        r["trace"]["watches"][1]["identity"]["binding_id"]
    );
    assert!(
        f.command()
            .args(["build", "app.fpasprj"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let mut args = vec!["debug", "app.fpascp", "--source-root", "."];
    args.extend(selectors);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
    let mut args = vec!["debug", "main.fpas"];
    args.extend(selectors);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
}

#[test]
fn unsupported_capture_requests_are_atomic_before_initializer_effects() {
    let f = Fixture::new(
        "program T; uses Std.Io.Fs as Fs; function Init(): integer; begin discard Fs.WriteText('executed', 'bad').Unwrap(); return 0; end function; var Seed: integer := Init(); function Make(Items: array of integer; Callback: function(): integer): function(): integer; function Read(): integer; begin return Items[0] + Callback(); end function; begin return Read; end function; begin null; end.",
    );
    for bad in [
        "Make.Read::Items",
        "Make.Read::Callback",
        "Make.Read::Items[Missing]",
        "Make.Read::Items[0]()",
        "Read::Items[0]",
    ] {
        let r = f.run(
            &[
                "debug",
                "main.fpas",
                "--trace",
                "Make.Read::Items[0]",
                "--trace",
                bad,
            ],
            1,
        );
        assert_eq!(r["trace"]["reasons"], json!(["binding_failed"]));
        assert!(r["trace"]["watches"].as_array().unwrap().is_empty());
        assert!(!f.0.join("executed").exists());
    }
}

#[test]
fn seeded_worker_capture_histories_are_repeatable_and_report_actual_writers() {
    let f = Fixture::new(
        "program T; uses Std.Tasks as Tasks; function Work(Seed: integer): integer; function Add(N: integer): integer; begin Total := Total + N; return Total; end function; begin var Total := Seed; discard [1, 2].Map(Add); return Total; end function; begin scope const A := go Work(1); const B := go Work(10); discard Tasks.Wait(A); discard Tasks.Wait(B); end scope; end.",
    );
    let args = [
        "debug",
        "main.fpas",
        "--trace",
        "Work.Add::Total",
        "--trace",
        "Work::Total",
        "--schedule-seed",
        "42",
    ];
    let r = f.run(&args, 0);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
    for (task, expected) in [
        (1, vec![json!("1"), json!("2"), json!("2"), json!("4")]),
        (2, vec![json!("10"), json!("11"), json!("11"), json!("13")]),
    ] {
        let events: Vec<_> = r["trace"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["watch_id"] == 0 && e["task_id"] == task)
            .collect();
        assert_eq!(
            events
                .iter()
                .map(|e| e["new_value"]["scalar"].clone())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(events.iter().all(|e| e["function"] == "work.add"));
    }
    assert_eq!(r["trace"]["complete"], true);
}

#[test]
fn suspended_capture_result_commits_before_breakpoint() {
    let f = Fixture::new(
        "program T;\nuses Std.Tasks as Tasks, Std.Time as Time;\nfunction Job(): integer;\nbegin Time.Sleep(10); return 7; end function;\nfunction Make(): function(): integer;\n function Next(): integer;\n begin\n  scope\n   const H := go Job();\n   Total := Tasks.Wait(H);\n  end scope;\n  Total := 8;\n  return Total;\n end function;\nbegin var Total := 0; return Next; end function;\nbegin const A := Make(); discard A(); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Make.Next::Total",
            "--schedule-seed",
            "42",
            "--break",
            "main.fpas:12",
            "--print",
            "Total",
        ],
        0,
    );
    assert_eq!(r["outcome"], "breakpoint");
    assert_eq!(values(&r, 0), vec![json!("0"), json!("7")]);
    assert_eq!(r["trace"]["events"][1]["location"]["line"], 10);
    assert_eq!(r["trace"]["complete"], true);
}

#[test]
fn delayed_reduce_result_keeps_capture_live_through_lexical_block_end() {
    let f = Fixture::new(
        "program T;\nfunction Add(A: integer; B: integer): integer;\nbegin return A + B; end function;\nfunction Make(): function(): integer;\n function Next(): integer;\n begin\n  begin\n   Total := [1, 2].Reduce(0, Add);\n  end;\n  return Total;\n end function;\nbegin var Total := 0; return Next; end function;\nbegin const A := Make(); discard A(); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Make.Next::Total"], 0);
    assert_eq!(values(&r, 0), vec![json!("0"), json!("3")]);
    assert_eq!(r["trace"]["events"][1]["location"]["line"], 8);
    assert_eq!(r["trace"]["events"][1]["function"], "make.next");
}

#[test]
fn failed_callback_and_event_limit_retain_complete_earliest_events() {
    let f = Fixture::new(
        "program T; function Make(): function(N: integer): integer; function Fail(N: integer): integer; begin Total := N; if N = 2 then panic('stop'); end if; return Total; end function; begin var Total := 0; return Fail; end function; begin const A := Make(); discard [1, 2, 3].Map(A); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Make.Fail::Total"], 2);
    assert_eq!(
        values(&r, 0),
        vec![json!("0"), json!("1"), json!("1"), json!("2")]
    );
    assert_eq!(r["trace"]["complete"], true);
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Make.Fail::Total",
            "--trace-limit",
            "3",
        ],
        3,
    );
    assert_eq!(r["trace"]["event_count"], 3);
    assert_eq!(r["trace"]["total_count"], 4);
    assert_eq!(r["trace"]["complete"], false);
}

#[test]
fn immutable_capture_projects_only_selected_scalar_from_large_record() {
    let f = Fixture::new(
        "program T; type Model = record X: integer; Payload: string; end record; function Make(State: Model): function(): integer; function Read(): integer; begin discard [1, 2].Map(function(N: integer): integer begin return N; end function); return State.X; end function; begin return Read; end function; begin const A := Make(Model(X := 7, Payload := '123456789'.RepeatStr(1000000))); discard A(); discard A(); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Make.Read::State.X"], 0);
    assert_eq!(values(&r, 0), vec![json!("7"), json!("7")]);
    assert_eq!(r["trace"]["complete"], true);
}
