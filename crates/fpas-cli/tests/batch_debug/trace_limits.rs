//! Trace limits must stop bounded execution without losing earlier complete events.

use super::*;

#[test]
fn exact_capacity_succeeds_and_next_change_reports_an_omitted_event() {
    let f = Fixture::new("program Capacity; var Total: integer := 1; begin Total := 2; end.");
    let args = [
        "debug",
        "main.fpas",
        "--trace",
        "Total",
        "--trace-limit",
        "2",
    ];
    let r = f.run(&args, 0);
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(r["trace"]["event_count"], 2);
    f.write(
        "main.fpas",
        "program Capacity; var Total: integer := 1; begin Total := 2; Total := 3; Total := 4; end.",
    );
    let r = f.run(&args, 3);
    assert_eq!(r["outcome"], "limit");
    assert_eq!(r["trace"]["complete"], false);
    assert_eq!(r["trace"]["event_count"], 2);
    assert_eq!(r["trace"]["events"][1]["new_value"]["scalar"], "2");
    assert!(
        r["trace"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("trace_event_limit"))
    );
    assert_eq!(r["capture"]["incomplete"], true);
}

#[test]
fn string_comparison_is_exact_past_the_display_prefix() {
    let f = Fixture::new(
        "program LongText; var Text: string := ''; begin Text := 'x'.RepeatStr(5000) + 'a'; Text := 'x'.RepeatStr(5000) + 'b'; end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Text"], 0);
    assert_eq!(r["trace"]["event_count"], 3);
    assert_eq!(r["trace"]["complete"], true);
    assert_eq!(
        r["trace"]["events"][1]["new_value"]["scalar"],
        r["trace"]["events"][2]["new_value"]["scalar"]
    );
    assert_eq!(r["trace"]["events"][2]["new_value"]["status"], "truncated");
    assert!(
        r["trace"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("value_truncated"))
    );
}

#[test]
fn oversized_comparison_values_end_execution_before_later_statements() {
    let f = Fixture::new(
        "program Huge; uses Std.Fs as Fs; var Text: string := ''; begin Text := 'x'.RepeatStr(65537); discard Fs.WriteText('executed','bad').Unwrap(); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--trace", "Text"], 3);
    assert_eq!(r["outcome"], "limit");
    assert_eq!(r["trace"]["event_count"], 1);
    assert_eq!(r["trace"]["complete"], false);
    assert_eq!(r["trace"]["total_count"], Value::Null);
    assert!(
        r["trace"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("trace_value_limit"))
    );
    assert!(!f.0.join("executed").exists());
}

#[test]
fn comparison_work_and_escaped_trace_bytes_have_separate_limits() {
    for (source, reason) in [
        (
            "program Work; var Text: string := ''; begin Text := 'x'.RepeatStr(65536); for I: integer := 0 to 300 do Text := Text; end for; end.",
            "trace_work_limit",
        ),
        (
            "program Bytes; var Text: string := ''; begin for I: integer := 0 to 100 do if I mod 2 = 0 then Text := (#1).RepeatStr(4096) + 'a'; else Text := (#1).RepeatStr(4096) + 'b'; end if; end for; end.",
            "trace_byte_limit",
        ),
    ] {
        let f = Fixture::new(source);
        let r = f.run(&["debug", "main.fpas", "--trace", "Text"], 3);
        assert_eq!(r["outcome"], "limit");
        assert_eq!(r["trace"]["complete"], false);
        assert!(
            r["trace"]["reasons"]
                .as_array()
                .unwrap()
                .contains(&Value::from(reason)),
            "{}",
            r["trace"]
        );
    }
}

#[test]
fn near_full_snapshots_with_prints_and_trace_remain_bounded() {
    let f = Fixture::new(
        "program Full; var Values: array of string := []; var Total: integer := 1; begin for I: integer := 0 to 63 do Values.PushInPlace((#1).RepeatStr(4096)); end for; Total := 2; end.",
    );
    let mut args = vec!["debug", "main.fpas", "--trace", "Total"];
    for _ in 0..8 {
        args.extend(["--print", "Values"]);
    }
    let r = f.run(&args, 3);
    assert_eq!(r["outcome"], "completed");
    assert_eq!(r["trace"]["event_count"], 2);
    assert_eq!(r["trace"]["complete"], true);
    assert!(
        r["expressions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| !v["value"].is_null())
    );
    assert_eq!(r["capture"]["incomplete"], true);
}

#[test]
fn instruction_limit_retains_published_history() {
    let f = Fixture::new(
        "program Loop; var Total: integer := 1; begin Total := 2; while true do null; end while; end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--instruction-limit",
            "1000",
        ],
        3,
    );
    assert_eq!(r["outcome"], "limit");
    assert_eq!(r["trace"]["event_count"], 2);
    assert_eq!(r["trace"]["complete"], true);
}

#[cfg(unix)]
#[test]
fn forced_worker_death_cannot_claim_complete_empty_history() {
    let f = Fixture::new(
        "program Die; uses Std.Proc as Proc; var Total: integer := 1; begin Total := 2; discard Proc.Run('sh', ['-c', 'kill -KILL $PPID']).Unwrap(); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--trace-limit",
            "17",
        ],
        3,
    );
    assert_eq!(r["trace"]["requested"], serde_json::json!(["Total"]));
    assert_eq!(r["trace"]["complete"], false);
    assert_eq!(r["trace"]["event_count"], 0);
    assert_eq!(r["trace"]["total_count"], Value::Null);
    assert!(
        r["trace"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("state_unavailable"))
    );
    assert_eq!(r["limits"]["trace_events"], 17);
}

#[cfg(unix)]
#[test]
fn blocked_preparation_and_native_calls_preserve_request_and_cleanup() {
    let f = Fixture::new(
        "program Blocked; uses Std.Time as Time; var Total: integer := 1; begin Time.Sleep(60000); end.",
    );
    let r = f.run(
        &["debug", "main.fpas", "--trace", "Total", "--timeout", "1"],
        3,
    );
    assert_eq!(r["trace"]["complete"], false);
    assert_eq!(r["trace"]["total_count"], Value::Null);
    let fifo = f.0.join("blocked.fpas");
    nix::unistd::mkfifo(
        &fifo,
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .unwrap();
    let r = f.run(
        &[
            "debug",
            "blocked.fpas",
            "--trace",
            "Total",
            "--timeout",
            "1",
        ],
        3,
    );
    assert_eq!(r["trace"]["requested"], serde_json::json!(["Total"]));
    assert_eq!(r["trace"]["complete"], false);
}

#[test]
fn output_limit_after_a_change_explains_any_unpublished_history() {
    let f = Fixture::new(
        "program Output; uses Std.Console as Console; var Total: integer := 1; begin Total := 2; Console.WriteText('x'.RepeatStr(2000)); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Total",
            "--output-limit",
            "10",
        ],
        3,
    );
    assert_eq!(r["outcome"], "limit");
    assert_eq!(r["trace"]["requested"], serde_json::json!(["Total"]));
    if r["trace"]["complete"] == true {
        assert_eq!(r["trace"]["event_count"], 2);
    } else {
        assert!(
            r["trace"]["reasons"]
                .as_array()
                .unwrap()
                .contains(&Value::from("state_unavailable"))
        );
        assert_eq!(r["trace"]["total_count"], Value::Null);
    }
    assert!(text(&r).len() <= 10);
}
