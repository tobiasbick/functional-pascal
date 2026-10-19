//! Real-process trace behavior through the same target and single-result path.

use super::*;

fn scalars(report: &Value) -> Vec<Value> {
    report["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["new_value"]["scalar"].clone())
        .collect()
}

#[test]
fn trace_collects_initialization_and_every_unequal_store_in_request_order() {
    let f = Fixture::new(
        "program Changes; var Total: integer := 1; begin Total := 1; Total := 2; Total := 1; end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--trace",
            "TOTAL",
            "--print",
            "Total",
        ],
        0,
    );
    assert_eq!(r["trace"]["requested"], serde_json::json!(["Total"]));
    assert_eq!(r["trace"]["watches"].as_array().unwrap().len(), 1);
    assert_eq!(
        scalars(&r),
        serde_json::json!(["1", "2", "1"])
            .as_array()
            .unwrap()
            .clone()
    );
    let events = r["trace"]["events"].as_array().unwrap();
    assert_eq!(events[0]["kind"], "initialize");
    assert_eq!(events[0]["old_value"]["status"], "uninitialized");
    assert_eq!(events[1]["old_value"]["scalar"], "1");
    for (index, event) in events.iter().enumerate() {
        assert_eq!(event["sequence"], index + 1);
        assert_eq!(event["watch_id"], 0);
        assert_eq!(event["task_id"], 0);
    }
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["trace"]["event_count"], 3);
    assert_eq!(r["trace"]["total_count"], 3);
    assert_eq!(r["expressions"][0]["value"]["scalar"], "1");
}

#[test]
fn fields_track_root_replacement_and_var_writes_without_unrelated_events() {
    let f = Fixture::new(
        "program Fields; type Model = record Selected: integer; Other: integer; end record; var State: Model := Model(Selected := 1, Other := 9); procedure SetValue(var Value: integer); begin Value := 4; end procedure; begin State.Other := 10; State := Model(Selected := 2, Other := 11); SetValue(var State.Selected); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "State.Selected",
            "--print",
            "State.Selected",
        ],
        0,
    );
    assert_eq!(
        scalars(&r),
        serde_json::json!(["1", "2", "4"])
            .as_array()
            .unwrap()
            .clone()
    );
    assert_eq!(r["expressions"][0]["value"]["scalar"], "4");
    assert_eq!(r["trace"]["events"][2]["kind"], "change");
}

#[test]
fn trace_source_belongs_to_writing_instruction_and_callback_function() {
    let f = Fixture::new(
        "program Callback;\nvar Total: integer := 0;\nfunction Map(C: string): string;\nbegin\n Total := Total + 1;\n return C;\nend function;\nbegin\n discard 'ab'.Map(Map);\nend.\n",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Total"], 0);
    assert_eq!(
        scalars(&r),
        serde_json::json!(["0", "1", "2"])
            .as_array()
            .unwrap()
            .clone()
    );
    for event in r["trace"]["events"].as_array().unwrap().iter().skip(1) {
        assert_eq!(event["location"]["line"], 5);
        assert!(
            event["function"]
                .as_str()
                .unwrap()
                .to_ascii_lowercase()
                .contains("map")
        );
    }
}

#[test]
fn breakpoint_prefix_runtime_failure_and_bad_inspection_keep_trace_coverage() {
    let f = Fixture::new(
        "program Stops;\nvar Total: integer := 1;\nbegin\n Total := 2;\n Total := 3;\n panic('failure');\nend.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--break",
            "main.fpas:5",
            "--print",
            "Total",
        ],
        0,
    );
    assert_eq!(
        scalars(&r),
        serde_json::json!(["1", "2"]).as_array().unwrap().clone()
    );
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["expressions"][0]["value"]["scalar"], "2");
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--print",
            "Missing",
        ],
        2,
    );
    assert_eq!(
        scalars(&r),
        serde_json::json!(["1", "2", "3"])
            .as_array()
            .unwrap()
            .clone()
    );
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["diagnostics"][0]["code"], "FP5010");
    assert!(!r["expressions"][0]["error"].is_null());
}

#[test]
fn request_errors_never_run_initializers_or_the_first_statement() {
    let f = Fixture::new(
        "program Guard; uses Std.Fs as Fs; const Fixed: integer := 1; type Model = record Item: integer; end record; function Initialize(): integer; begin discard Fs.WriteText('initialized','bad').Unwrap(); return 0; end function; var Total: integer := Initialize(); var State: Model := Model(Item := 1); var Items: array of integer := []; begin discard Fs.WriteText('executed','bad').Unwrap(); end.",
    );
    let check = f.command().args(["check", "main.fpas"]).output().unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    for options in [
        vec!["--trace", "Missing"],
        vec!["--trace", "Fixed"],
        vec!["--trace", "State"],
        vec!["--trace", "Items"],
        vec!["--trace", "Items[Missing]"],
        vec!["--trace", "Total()"],
        vec!["--trace", "Total", "--trace-limit", "0"],
        vec!["--trace", "Total", "--trace-limit", "10001"],
        vec!["--trace-limit", "1"],
        vec![
            "--trace",
            "Total",
            "--trace-limit",
            "1",
            "--trace-limit",
            "2",
        ],
        vec!["--trace", "Total", "--protocol", "jsonl"],
        vec!["--trace", "Total", "--protocol", "dap"],
    ] {
        let mut args = vec!["debug", "main.fpas"];
        args.extend(options);
        let r = f.run(&args, 1);
        assert_eq!(r["trace"]["event_count"], 0);
        assert!(!f.0.join("executed").exists());
        assert!(!f.0.join("initialized").exists());
    }
}

#[test]
fn trace_binding_is_atomic_and_empty_envelope_is_always_present() {
    let f = Fixture::new(HELLO);
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--trace",
            "Missing",
        ],
        1,
    );
    assert!(r["trace"]["watches"].as_array().unwrap().is_empty());
    assert_eq!(r["trace"]["reasons"], serde_json::json!(["binding_failed"]));
    let r = f.run(&["debug", "main.fpas"], 0);
    assert_eq!(
        r["trace"],
        serde_json::json!({"requested":[],"watches":[],"events":[],"complete":true,"reasons":[],"event_count":0,"total_count":0})
    );
    let r = f.run(
        &[
            "debug",
            "missing.fpas",
            "--trace",
            "Total",
            "--trace-limit",
            "17",
        ],
        1,
    );
    assert_eq!(r["trace"]["requested"], serde_json::json!(["Total"]));
    assert_eq!(r["limits"]["trace_events"], 17);
    assert_eq!(r["trace"]["complete"], false);
    assert_eq!(r["trace"]["total_count"], Value::Null);
}

#[test]
fn project_workspace_discovery_and_verified_images_share_trace_binding() {
    let f = Fixture::new(HELLO);
    f.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n");
    f.write(
        "suite.fpasworkspace",
        "[workspace]\nname = 'suite'\nmembers = ['app.fpasprj']\n",
    );
    for target in [None, Some("app.fpasprj"), Some("suite.fpasworkspace")] {
        let mut args = vec!["debug"];
        args.extend(target);
        args.extend(["--trace", "Total"]);
        assert_eq!(f.run(&args, 0)["trace"]["event_count"], 2);
    }
    let build = f.command().args(["build", "app.fpasprj"]).output().unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let r = f.run(
        &[
            "debug",
            "app.fpascp",
            "--source-root",
            ".",
            "--trace",
            "Total",
        ],
        0,
    );
    assert_eq!(r["trace"]["event_count"], 2);
}

#[test]
fn scripted_input_and_program_arguments_are_visible_as_changes() {
    let f = Fixture::new(
        "program Input; uses Std.Args as Args, Std.Console as Console; var Text: string := ''; begin Text := Args.ParamStr(0); Text := Console.ReadLn(); Text := Console.ReadKey(); end.",
    );
    f.write(
        "input.toml",
        "[[event]]\ntype='readln'\nline='line'\n[[event]]\ntype='readkey_chars'\nchars='x'\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Text",
            "--script",
            "input.toml",
            "--",
            "argument",
        ],
        0,
    );
    assert_eq!(
        scalars(&r),
        serde_json::json!(["", "argument", "line", "x"])
            .as_array()
            .unwrap()
            .clone()
    );
}

#[test]
fn unit_globals_use_the_full_metadata_name_and_reject_import_aliases() {
    let f = Fixture::new("program Main; uses WatchState as W; begin W.Increment(); end.");
    f.write("WatchState.fpas", "unit WatchState; var Total: integer := 1; public procedure Increment(); begin Total := 2; end procedure; end unit;");
    f.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let r = f.run(&["debug", "app.fpasprj", "--trace", "WatchState.Total"], 0);
    assert_eq!(
        scalars(&r),
        serde_json::json!(["1", "2"]).as_array().unwrap().clone()
    );
    let r = f.run(&["debug", "app.fpasprj", "--trace", "W.Total"], 1);
    assert_eq!(r["trace"]["reasons"], serde_json::json!(["binding_failed"]));
}

#[test]
fn selector_and_option_occurrence_bounds_are_checked_before_execution() {
    let f = Fixture::new(HELLO);
    let mut args = vec!["debug", "main.fpas"];
    for _ in 0..65 {
        args.extend(["--trace", "Total"]);
    }
    f.run(&args, 1);
    for selector in ["".into(), "x".repeat(4097)] {
        f.run(&["debug", "main.fpas", "--trace", &selector], 1);
    }
}

#[test]
fn a_watch_not_initialized_before_the_first_breakpoint_has_complete_empty_history() {
    let f = Fixture::new(
        "program Early;\nfunction Seed(): integer;\nbegin\n return 1;\nend function;\nvar First: integer := Seed();\nvar Later: integer := 2;\nbegin null; end.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Later",
            "--break",
            "main.fpas:4",
        ],
        0,
    );
    assert_eq!(r["trace"]["watches"].as_array().unwrap().len(), 1);
    assert_eq!(r["trace"]["event_count"], 0);
    assert_eq!(r["trace"]["total_count"], 0);
    assert_eq!(r["trace"]["complete"], true);
}

#[test]
fn missed_breakpoints_and_final_inspection_errors_do_not_mark_history_lost() {
    let f = Fixture::new(
        "program Missing;\nvar Total: integer := 1;\nfunction Never(): integer;\nbegin\n return 9;\nend function;\nbegin Total := 2; end.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--break",
            "main.fpas:5",
            "--print",
            "Unknown",
        ],
        3,
    );
    assert_eq!(r["trace"]["event_count"], 2);
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["diagnostics"][0]["code"], "breakpoint_not_hit");
    assert!(!r["expressions"][0]["error"].is_null());
}
