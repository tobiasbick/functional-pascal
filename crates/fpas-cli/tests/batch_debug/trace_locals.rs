//! Source-to-process coverage for exact local selectors and invocation lifetimes.

use super::*;
use serde_json::json;

const LOCAL: &str = "program Local; function F(N: integer): integer; begin var Total: integer := N; Total := Total + 1; return Total; end function; begin discard F(1); discard F(4); end.";

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
fn exact_local_selectors_deduplicate_and_keep_call_and_declaration_identities() {
    let f = Fixture::new(LOCAL);
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "F::Total",
            "--trace",
            "f::total",
            "--trace",
            "F::N",
        ],
        0,
    );
    assert_eq!(r["trace"]["requested"], json!(["F::Total", "F::N"]));
    assert_eq!(
        values(&r, 0),
        vec![json!("1"), json!("2"), json!("4"), json!("5")]
    );
    assert_eq!(values(&r, 1), vec![json!("1"), json!("4")]);
    assert_eq!(r["trace"]["watches"][0]["identity"]["kind"], "local");
    assert_eq!(r["trace"]["watches"][1]["identity"]["kind"], "parameter");
    let events = r["trace"]["events"].as_array().unwrap();
    assert_eq!(events[0]["activation_id"], events[1]["activation_id"]);
    assert_eq!(events[1]["instance_id"], events[2]["instance_id"]);
    assert_ne!(events[0]["activation_id"], events[3]["activation_id"]);
    assert_eq!(r["limits"]["trace_live_instances"], 1024);
    assert_eq!(r["limits"]["trace_local_retained_value_bytes"], 8388608);
}

#[test]
fn project_and_portable_image_preserve_local_metadata_and_results() {
    let f = Fixture::new(LOCAL);
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let source = f.run(&["debug", "app.fpasprj", "--trace", "F::Total"], 0);
    let build = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let image = f.run(
        &[
            "debug",
            "app.fpascp",
            "--source-root",
            ".",
            "--trace",
            "F::Total",
        ],
        0,
    );
    assert_eq!(source["trace"], image["trace"]);
}

#[test]
fn parameter_admission_is_visible_at_first_callee_breakpoint() {
    let f = Fixture::new(
        "program Entry;\nprocedure F(N: integer);\nbegin\n var Total: integer := N;\n Total := 2;\nend procedure;\nbegin F(7); end.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "F::N",
            "--trace",
            "F::Total",
            "--break",
            "main.fpas:4",
        ],
        0,
    );
    assert_eq!(r["outcome"], "breakpoint");
    assert_eq!(values(&r, 0), vec![json!("7")]);
    assert!(values(&r, 1).is_empty());
    assert_eq!(r["trace"]["complete"], true);
}

#[test]
fn local_binding_failure_is_atomic_before_global_initializers() {
    let f = Fixture::new(
        "program Guard; uses Std.Io.Fs as Fs; function Init(): integer; begin discard Fs.WriteText('executed', 'bad').Unwrap(); return 0; end function; var Total: integer := Init(); procedure F(N: integer); begin var X: integer := N; begin var X: integer := 2; end; const Items := [1]; end procedure; begin F(1); end.",
    );
    for target in [
        "F::X",
        "F::Items",
        "F::N.Bad",
        "F::N[0]",
        "F::N()",
        "Missing::N",
    ] {
        let r = f.run(
            &["debug", "main.fpas", "--trace", "Total", "--trace", target],
            1,
        );
        assert_eq!(
            r["trace"]["reasons"],
            json!(["binding_failed"]),
            "{target}: {r}"
        );
        assert!(r["trace"]["watches"].as_array().unwrap().is_empty());
        assert!(!f.0.join("executed").exists());
    }
}

#[test]
fn global_and_parameter_aliases_keep_request_order_and_failed_store_prefix() {
    let f = Fixture::new(
        "program Alias; var Total: integer := 0; procedure Change(var N: integer); begin N := 1; N := 2; N := 1 div 0; end procedure; begin Change(var Total); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Change::N",
            "--trace",
            "Total",
        ],
        2,
    );
    assert_eq!(values(&r, 0), vec![json!("0"), json!("1"), json!("2")]);
    let changes: Vec<_> = r["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "change")
        .map(|e| e["watch_id"].clone())
        .collect();
    assert_eq!(changes, vec![json!(0), json!(1), json!(0), json!(1)]);
    assert_eq!(r["trace"]["events"][0]["activation_id"], Value::Null);
    assert_eq!(r["trace"]["complete"], true);
    let limited = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Change::N",
            "--trace-limit",
            "2",
        ],
        3,
    );
    assert_eq!(limited["trace"]["event_count"], 2);
    assert_eq!(limited["trace"]["total_count"], 3);
    assert_eq!(limited["diagnostics"][0]["code"], "trace_event_limit");
}

#[test]
fn seeded_worker_callbacks_keep_task_ids_and_repeated_call_instances() {
    let f = Fixture::new(
        "program Tasks; uses Std.Tasks as Tasks; function Map(N: integer): integer; begin var Total: integer := N; Total := Total + 1; return Total; end function; function Work(): integer; begin return [1, 2, 1].Map(Map).Length(); end function; begin scope const A := go Work(); const B := go Work(); discard Tasks.Wait(A); discard Tasks.Wait(B); end scope; end.",
    );
    let args = [
        "debug",
        "main.fpas",
        "--trace",
        "Map::Total",
        "--trace",
        "Map::N",
        "--schedule-seed",
        "42",
    ];
    let r = f.run(&args, 0);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
    assert_eq!(r["trace"]["event_count"], 18);
    let events = r["trace"]["events"].as_array().unwrap();
    let tasks: std::collections::BTreeSet<_> = events
        .iter()
        .map(|e| e["task_id"].as_u64().unwrap())
        .collect();
    assert_eq!(tasks, [1, 2].into_iter().collect());
    let ids: std::collections::BTreeSet<_> = events
        .iter()
        .filter(|e| e["watch_id"] == 1)
        .map(|e| e["activation_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 6);
    assert_eq!(r["trace"]["complete"], true);
}

#[test]
fn nested_callback_writes_are_attributed_to_the_writing_function() {
    let f = Fixture::new(
        "program Callbacks; procedure Outer(); begin var Total: integer := 0; discard [1, 2, 1].Map(function(N: integer): integer begin discard [N].Map(function(M: integer): integer begin Total := M; return M; end function); return N; end function); end procedure; begin Outer(); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Outer::Total"], 0);
    assert_eq!(
        values(&r, 0),
        vec![json!("0"), json!("1"), json!("2"), json!("1")]
    );
    for event in r["trace"]["events"].as_array().unwrap().iter().skip(1) {
        assert_ne!(
            event["function"].as_str().unwrap().to_ascii_lowercase(),
            "outer"
        );
        assert_eq!(event["task_id"], 0);
    }
}

#[test]
fn worker_callback_results_and_writes_survive_the_enclosing_block_end() {
    for (body, expected, result_assignment) in [
        (
            "Total := [1, 2].Reduce(0, Add);",
            vec![json!("0"), json!("3")],
            true,
        ),
        (
            "discard [1, 2, 3].Map(function(N: integer): integer begin Total := N; return N; end function);",
            vec![json!("0"), json!("1"), json!("2"), json!("3")],
            false,
        ),
    ] {
        let f = Fixture::new(&format!(
            "program T;\nuses Std.Tasks as Tasks;\nfunction Add(A: integer; B: integer): integer;\nbegin\n return A + B;\nend function;\nfunction Work(): integer;\nbegin\n begin\n var Total: integer := 0;\n {body}\n end;\n return 0;\nend function;\nbegin scope const Job := go Work(); discard Tasks.Wait(Job); end scope; end."
        ));
        let r = f.run(&["debug", "main.fpas", "--trace", "Work::Total"], 0);
        assert_eq!(values(&r, 0), expected);
        assert_eq!(r["trace"]["complete"], true);
        for event in r["trace"]["events"].as_array().unwrap().iter().skip(1) {
            assert_eq!(event["task_id"], 1);
            assert_eq!(event["location"]["line"], 11);
            assert_eq!(event["function"] == "work", result_assignment);
        }
    }
}

#[test]
fn suspension_result_commits_precede_breakpoints_and_keep_the_wait_source() {
    let f = Fixture::new(
        "program T;\nuses Std.Tasks as Tasks, Std.Time as Time;\nfunction Job(): integer;\nbegin\n Time.Sleep(10);\n return 7;\nend function;\nprocedure Outer();\nbegin\n var Total: integer := 0;\n scope\n const Handle := go Job();\n Total := Tasks.Wait(Handle);\n Total := Total + 1;\n end scope;\nend procedure;\nbegin Outer(); end.",
    );
    let base = [
        "debug",
        "main.fpas",
        "--trace",
        "Outer::Total",
        "--schedule-seed",
        "42",
    ];
    let r = f.run(&base, 0);
    assert_eq!(values(&r, 0), vec![json!("0"), json!("7"), json!("8")]);
    assert_eq!(r["trace"]["events"][1]["location"]["line"], 13);
    let mut args = base.to_vec();
    args.extend(["--break", "main.fpas:14"]);
    let r = f.run(&args, 0);
    assert_eq!(r["outcome"], "breakpoint");
    assert_eq!(values(&r, 0), vec![json!("0"), json!("7")]);
    assert_eq!(r["trace"]["complete"], true);
    args.extend(["--trace-limit", "1"]);
    let r = f.run(&args, 3);
    assert_eq!(r["diagnostics"][0]["code"], "trace_event_limit");
    assert_eq!(values(&r, 0), vec![json!("0")]);
    assert_eq!(r["trace"]["total_count"], 2);
}

#[test]
fn nominal_parameter_and_local_types_survive_portable_image_storage() {
    let f = Fixture::new(
        "program T; type Mood = enum Sad; Glad; end enum; type Limit = 1..10; type Dollars = distinct integer; procedure F(V: Mood; S: Limit; D: Dollars); function ChangeCash(N: integer): integer; begin Cash := Dollars(N); return N; end function; begin const State: Mood := V; const Small := S; var Cash: Dollars := D; discard [1].Map(ChangeCash); end procedure; begin F(Mood.Glad, Limit(3), Dollars(5)); end.",
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let args = [
        "--trace", "F::V", "--trace", "F::State", "--trace", "F::Small", "--trace", "F::Cash",
    ];
    let mut source_args = vec!["debug", "app.fpasprj"];
    source_args.extend(args);
    let r = f.run(&source_args, 0);
    for (index, expected) in ["Mood", "Mood", "Limit", "Dollars"].into_iter().enumerate() {
        assert_eq!(r["trace"]["watches"][index]["type"], expected);
        let event = r["trace"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["watch_id"] == index)
            .unwrap();
        assert_eq!(event["new_value"]["type"], expected);
    }
    assert_eq!(values(&r, 3), vec![json!("5"), json!("1")]);
    assert!(
        f.command()
            .args(["build", "app.fpasprj"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let mut image_args = vec!["debug", "app.fpascp", "--source-root", "."];
    image_args.extend(args);
    assert_eq!(r["trace"], f.run(&image_args, 0)["trace"]);
}
