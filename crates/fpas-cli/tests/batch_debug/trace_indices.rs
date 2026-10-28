//! Indexed scalar paths preserve known absence across CLI, image, stop and bound paths.

use super::*;
use serde_json::json;

fn values(r: &Value, watch: usize) -> Vec<Value> {
    r["trace"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["watch_id"] == watch)
        .map(|e| e["new_value"]["scalar"].clone())
        .collect()
}

#[test]
fn functional_and_mutating_array_operations_keep_position_history_in_images() {
    let f = Fixture::new(
        "program T; procedure Update(Input: array of integer); begin var Items := Input; Items := Items.Push(3); Items.SortInPlace(); Items := Items.Pop().Remaining; Items := []; end procedure; begin Update([2, 1]); end.",
    );
    f.write("app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['main.fpas']\n");
    let selects = [
        "--trace",
        "Update::Items[1]",
        "--trace",
        "Update::Input[1]",
        "--trace",
        "Update::Items[99]",
    ];
    let mut args = vec!["debug", "app.fpasprj"];
    args.extend(selects);
    let r = f.run(&args, 0);
    assert_eq!(values(&r, 0), vec![json!("1"), json!("2"), json!(null)]);
    assert_eq!(values(&r, 1), vec![json!("1")]);
    assert_eq!(values(&r, 2), vec![json!(null)]);
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["limits"]["trace_path_depth"], 16);
    assert!(
        f.command()
            .args(["build", "app.fpasprj"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let mut args = vec!["debug", "app.fpascp", "--source-root", "."];
    args.extend(selects);
    assert_eq!(r["trace"], f.run(&args, 0)["trace"]);
}

#[test]
fn invalid_index_requests_do_not_execute_any_initializer() {
    let f = Fixture::new(
        "program T; uses Std.Io.Fs as Fs; function Init(): integer; begin discard Fs.WriteText('executed', 'bad').Unwrap(); return 0; end function; var Seed: integer := Init(); procedure F(); begin var Items: array of integer := []; const Text: string := 'x'; end procedure; begin F(); end.",
    );
    for bad in [
        "F::Items[-1]",
        "F::Items[+1]",
        "F::Items[01]",
        "F::Items[ 0]",
        "F::Items[0 ]",
        "F::Items[N]",
        "F::Items[1+1]",
        "F::Items[0..1]",
        "F::Items[*]",
        "F::Items[9223372036854775808]",
        "F::Text[0]",
    ] {
        let r = f.run(
            &[
                "debug",
                "main.fpas",
                "--trace",
                "F::Items[0]",
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
fn delayed_array_result_precedes_breakpoint_and_trace_capacity_stop() {
    let f = Fixture::new(
        "program T;\nuses Std.Tasks as Tasks, Std.Time as Time;\nfunction Job(): array of integer;\nbegin Time.Sleep(10); return [7]; end function;\nprocedure Outer();\nbegin\n var Items: array of integer := [];\n scope const H := go Job();\n Items := Tasks.Wait(H);\n Items[0] := 8;\n end scope;\nend procedure;\nbegin Outer(); end.",
    );
    let base = [
        "debug",
        "main.fpas",
        "--trace",
        "Outer::Items[0]",
        "--schedule-seed",
        "42",
        "--break",
        "main.fpas:10",
    ];
    let r = f.run(&base, 0);
    assert_eq!(r["outcome"], "breakpoint");
    assert_eq!(values(&r, 0), vec![json!(null), json!("7")]);
    assert_eq!(r["trace"]["events"][1]["location"]["line"], 9);
    assert_eq!(
        r["trace"]["events"][1]["old_value"]["reason"],
        "index_out_of_bounds"
    );
    assert_eq!(r["trace"]["complete"], true);
    let mut args = base.to_vec();
    args.extend(["--trace-limit", "1"]);
    let r = f.run(&args, 3);
    assert_eq!(r["diagnostics"][0]["code"], "trace_event_limit");
    assert_eq!(r["trace"]["event_count"], 1);
    assert_eq!(r["trace"]["total_count"], 2);
}

#[test]
fn huge_absent_position_has_exact_identity_and_complete_coverage() {
    let f = Fixture::new(
        "program T; var Items: array of integer := []; begin Items := [1]; Items := []; end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Items[9223372036854775807]",
            "--trace-limit",
            "1",
        ],
        0,
    );
    assert_eq!(
        r["trace"]["events"][0]["new_value"]["reason"],
        "index_out_of_bounds"
    );
    assert_eq!(
        r["trace"]["watches"][0]["identity"]["path"][0]["index"],
        "9223372036854775807"
    );
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["trace"]["total_count"], 1);
    assert_eq!(r["capture"]["incomplete"], false);
}

#[test]
fn nested_index_field_callbacks_keep_actual_writer_and_block_end_coverage() {
    let f = Fixture::new(
        "program T; uses Std.Tasks as Tasks; type Row = record Value: integer; end record; function Work(): integer; begin begin var Items: array of Row := []; discard [1, 2, 3].Map(function(N: integer): integer begin Items := [Row(Value := N)]; return N; end function); end; return 0; end function; begin scope const H := go Work(); discard Tasks.Wait(H); end scope; end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Work::Items[0].Value",
            "--schedule-seed",
            "42",
        ],
        0,
    );
    assert_eq!(
        values(&r, 0),
        vec![json!(null), json!("1"), json!("2"), json!("3")]
    );
    for event in r["trace"]["events"].as_array().unwrap().iter().skip(1) {
        assert_eq!(event["task_id"], 1);
        assert_ne!(event["function"], "work");
    }
    assert_eq!(r["trace"]["complete"], true);
}
