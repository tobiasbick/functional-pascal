//! Record leaf tracepoints through metadata binding, process stops and portable images.

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
fn source_and_image_records_share_leaf_ids_types_and_deduplication() {
    let f = Fixture::new(
        "program T; type Count = distinct integer; type Model = record X: Count; Text: string; end record; procedure F(Input: Model); begin var State := Input; const Fixed := Input; State := Model(X := Count(2), Text := 'b'); State.X := Count(1); end procedure; begin F(Model(X := Count(1), Text := 'a')); end.",
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let selectors = [
        "--trace",
        "F::State.X",
        "--trace",
        "F::State.Text",
        "--trace",
        "F::Input.X",
        "--trace",
        "F::Fixed.X",
        "--trace",
        "f::state.x",
    ];
    let mut args = vec!["debug", "app.fpasprj"];
    args.extend(selectors);
    let r = f.run(&args, 0);
    assert_eq!(r["trace"]["watches"].as_array().unwrap().len(), 4);
    assert_eq!(r["trace"]["watches"][0]["type"], "Count");
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2"), json!("1")]);
    assert_eq!(values(&r, 1), vec![json!("a"), json!("b")]);
    assert_eq!(values(&r, 2), vec![json!("1")]);
    assert_eq!(values(&r, 3), vec![json!("1")]);
    let initial: Vec<_> = r["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "initialize" && (e["watch_id"] == 0 || e["watch_id"] == 1))
        .collect();
    assert_eq!(initial[0]["instance_id"], initial[1]["instance_id"]);
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
}

#[test]
fn invalid_field_request_is_atomic_before_any_initializer() {
    let f = Fixture::new(
        "program T; uses Std.Fs as Fs; type Model = record X: integer; end record; function Init(): integer; begin discard Fs.WriteText('executed', 'bad').Unwrap(); return 0; end function; var Seed: integer := Init(); procedure F(); begin var State := Model(X := 0); end procedure; begin F(); end.",
    );
    for bad in [
        "F::State",
        "F::State.NoSuch",
        "F::State.X.More",
        "F::State.X()",
        "F::State[0].X",
    ] {
        let r = f.run(
            &[
                "debug",
                "main.fpas",
                "--trace",
                "F::State.X",
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
fn suspended_record_result_is_traced_before_the_following_breakpoint() {
    let f = Fixture::new(
        "program T;\nuses Std.Tasks as Tasks, Std.Time as Time;\ntype Model = record X: integer; end record;\nfunction Job(): Model;\nbegin Time.Sleep(10); return Model(X := 7); end function;\nprocedure Outer();\nbegin\n var State := Model(X := 0);\n scope\n const H := go Job();\n State := Tasks.Wait(H);\n State.X := 8;\n end scope;\nend procedure;\nbegin Outer(); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Outer::State.X",
            "--schedule-seed",
            "42",
            "--break",
            "main.fpas:12",
        ],
        0,
    );
    assert_eq!(r["outcome"], "breakpoint");
    assert_eq!(values(&r, 0), vec![json!("0"), json!("7")]);
    assert_eq!(r["trace"]["events"][1]["location"]["line"], 11);
    assert_eq!(r["trace"]["complete"], true);
}

#[test]
fn failed_field_store_and_exact_capacity_preserve_complete_event_prefixes() {
    let f = Fixture::new(
        "program T; type Model = record X: integer; Y: integer; end record; procedure F(); begin var State := Model(X := 1, Y := 2); State.X := 2; State.X := 1 div 0; end procedure; begin F(); end.",
    );
    let args = [
        "debug",
        "main.fpas",
        "--trace",
        "F::State.X",
        "--trace",
        "F::State.Y",
    ];
    let r = f.run(&args, 2);
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2")]);
    assert_eq!(values(&r, 1), vec![json!("2")]);
    assert_eq!(r["trace"]["complete"], true);
    let mut limited = args.to_vec();
    limited.extend(["--trace-limit", "2"]);
    let r = f.run(&limited, 3);
    assert_eq!(r["trace"]["event_count"], 2);
    assert_eq!(r["trace"]["total_count"], 3);
    assert_eq!(r["diagnostics"][0]["code"], "trace_event_limit");
}

#[test]
fn readonly_record_parameters_retain_only_selected_scalars_with_large_other_fields() {
    let f = Fixture::new(
        "program T; type Model = record Flag: boolean; Payload: string; end record; procedure F(Input: Model); begin var N: integer := 0; while N < 1000 do N := N + 1; end while; end procedure; begin F(Model(Flag := true, Payload := '123456789'.RepeatStr(1000000))); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "F::Input.Flag"], 0);
    assert_eq!(values(&r, 0), vec![json!(true)]);
    assert_eq!(r["trace"]["complete"], true);
}
