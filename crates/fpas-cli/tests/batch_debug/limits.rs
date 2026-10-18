use super::*;
use std::time::{Duration, Instant};

#[test]
fn instruction_and_output_limits_return_explicit_partial_results() {
    let f = Fixture::new("program Loop; begin while true do null; end while; end.");
    let r = f.run(&["debug", "main.fpas", "--instruction-limit", "1000"], 3);
    assert_eq!(r["outcome"], "limit");
    f.write("main.fpas", "program Flood; uses Std.Console as Console; begin Console.WriteText('x'.RepeatStr(10000)); end.");
    let r = f.run(&["debug", "main.fpas", "--output-limit", "100"], 3);
    assert_eq!(r["outcome"], "limit");
    assert!(text(&r).len() <= 100);
    assert_eq!(r["capture"]["incomplete"], true);
}
#[test]
fn wall_clock_timeout_contains_a_blocked_native_call() {
    let f = Fixture::new("program Blocked; uses Std.Time as Time; begin Time.Sleep(60000); end.");
    let started = Instant::now();
    let r = f.run(&["debug", "main.fpas", "--timeout", "1"], 3);
    assert!(started.elapsed() < Duration::from_secs(8));
    assert_eq!(r["outcome"], "limit");
    assert!(
        r["capture"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("execution_timeout"))
    );
}
#[cfg(unix)]
#[test]
fn native_descendants_are_killed_with_the_worker() {
    let f = Fixture::new(
        "program Tree; uses Std.Proc as Proc; begin discard Proc.Run('sh', ['-c', '(sleep 2; printf leaked > escaped) & wait']).Unwrap(); end.",
    );
    f.run(&["debug", "main.fpas", "--timeout", "1"], 3);
    std::thread::sleep(Duration::from_millis(1200));
    assert!(!f.0.join("escaped").exists());
}
#[cfg(unix)]
#[test]
fn worker_death_still_yields_a_complete_error() {
    let f = Fixture::new(
        "program Die; uses Std.Proc as Proc; begin discard Proc.Run('sh', ['-c', 'kill -KILL $PPID']).Unwrap(); end.",
    );
    let r = f.run(&["debug", "main.fpas"], 3);
    assert_eq!(r["outcome"], "error");
    assert_eq!(r["capture"]["incomplete"], true);
}

#[cfg(unix)]
#[test]
fn deadline_also_covers_blocked_target_preparation() {
    let f = Fixture::new(HELLO);
    let fifo = f.0.join("blocked.fpas");
    nix::unistd::mkfifo(
        &fifo,
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .unwrap();
    let started = Instant::now();
    let r = f.run(&["debug", "blocked.fpas", "--timeout", "1"], 3);
    assert_eq!(r["outcome"], "limit");
    assert!(started.elapsed() < Duration::from_secs(8));
}
#[test]
fn escaped_values_stay_inside_the_complete_json_byte_bound() {
    let f = Fixture::new(
        "program Escaped; var Values: array of string := []; begin for I: integer := 0 to 63 do Values.PushInPlace((#1).RepeatStr(4096)); end for; end.",
    );
    let r = f.run(&["debug", "main.fpas", "--print", "Values"], 0);
    assert_eq!(r["outcome"], "completed");
    assert_eq!(
        r["expressions"][0]["value"]["children"]
            .as_array()
            .unwrap()
            .len(),
        64
    );
}

#[test]
fn a_giant_panic_message_cannot_replace_the_primary_runtime_failure() {
    let f = Fixture::new(
        "program Giant; begin const Chunk := 'x'.RepeatStr(1000000); panic(Chunk + Chunk + Chunk + Chunk + Chunk + Chunk + Chunk + Chunk + Chunk); end.",
    );
    let r = f.run(&["debug", "main.fpas", "--timeout", "10"], 2);
    assert_eq!(r["outcome"], "runtime_error");
    assert_eq!(r["diagnostics"][0]["code"], "FP5010");
    assert_eq!(r["diagnostics"][0]["truncated"], true);
    assert_eq!(r["capture"]["incomplete"], true);
    assert!(
        r["capture"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("byte_limit"))
    );
}

#[test]
fn escaped_snapshot_byte_budget_marks_requested_values_that_do_not_fit() {
    let f = Fixture::new(
        "program Escaped; var Values: array of string := []; begin for I: integer := 0 to 63 do Values.PushInPlace((#1).RepeatStr(4096)); end for; end.",
    );
    let mut args = vec!["debug", "main.fpas"];
    for _ in 0..8 {
        args.extend(["--print", "Values"]);
    }
    let r = f.run(&args, 3);
    assert_eq!(r["outcome"], "completed");
    assert_eq!(r["capture"]["incomplete"], true);
    assert!(
        r["capture"]["reasons"]
            .as_array()
            .unwrap()
            .contains(&Value::from("byte_limit"))
    );
    assert!(
        r["expressions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| !value["error"].is_null())
    );
    assert!(
        r["expressions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| !value["value"].is_null())
    );
}
