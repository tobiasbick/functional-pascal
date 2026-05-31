use super::{compile_and_run, compile_run_err};

#[test]
fn std_time_timestamp_and_monotonic_millis_are_positive() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Time;
begin
  WriteLn(TimestampMillis() > 0);
  WriteLn(MonotonicMillis() >= 0)
end.",
    );
    assert_eq!(out.lines, vec!["true", "true"]);
}

#[test]
fn std_time_elapsed_millis_measures_sleep() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Time;
begin
  var Start: integer := MonotonicMillis();
  Sleep(25);
  WriteLn(ElapsedMillis(Start) >= 10)
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn std_time_qualified_calls_work() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Time;
begin
  WriteLn(Std.Time.MonotonicMillis() >= 0)
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn std_time_sleep_rejects_negative_milliseconds() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Time;
begin
  Sleep(-1)
end.",
    );
    assert!(msg.contains("non-negative") || msg.contains("Sleep"));
}
