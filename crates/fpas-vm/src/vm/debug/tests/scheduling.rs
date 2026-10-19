//! Controlled real programs, including nested callbacks and virtual timers.

use super::*;

fn session(source: &str, seed: u64) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    DebugSession::with_limits_and_schedule(
        fpas_compiler::compile(&program).expect("compile seeded fixture"),
        vec!["fixed".into()],
        DebugInspectionLimits::default(),
        DebugExecutionLimits::default(),
        Some(seed),
    )
    .expect("seeded session")
}

const ORDERING: &str = r#"program Ordered;
uses Std.Tasks as Tasks, Std.Time as Time, Std.Console as Console;
procedure Produce(Q: channel of integer; Token: Tasks.CancellationToken; Value: integer);
begin
  const Ready := Tasks.CancellationCase(Token, procedure() begin end procedure);
  discard Tasks.Select([Ready]);
  discard Tasks.Send(Q, Value);
end procedure;
begin scope
  const Q: channel of integer := Tasks.CreateChannel(1);
  const Source := Tasks.CreateCancellationSource();
  const Token := Tasks.GetCancellationToken(Source);
  discard go Produce(Q, Token, 1);
  discard go Produce(Q, Token, 2);
  Time.Sleep(1000);
  discard Tasks.Cancel(Source);
  Console.WriteLn(Tasks.Receive(Q).Unwrap());
  Console.WriteLn(Tasks.Receive(Q).Unwrap());
end scope; end."#;

#[test]
fn seeded_programs_repeat_and_different_seeds_explore_channel_order() {
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..24 {
        let mut a = session(ORDERING, seed);
        let mut b = session(ORDERING, seed);
        assert!(matches!(
            a.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
        assert!(matches!(
            b.continue_execution().unwrap(),
            DebugRunResult::Terminated(_)
        ));
        assert_eq!(format!("{:?}", a.output()), format!("{:?}", b.output()));
        assert_eq!(a.schedule_report(), b.schedule_report());
        seen.insert(format!("{:?}", a.output()));
        assert_eq!(a.schedule_report().status, "controlled");
    }
    assert_eq!(
        seen.len(),
        2,
        "different seeds must actually change channel order"
    );
}

#[test]
fn virtual_time_jumps_and_cpu_peers_do_not_starve_timers() {
    let mut s = session(
        r#"program Timers;
uses Std.Tasks as Tasks, Std.Time as Time;
procedure Busy(Q: channel of integer);
begin
  var I: integer := 0;
  while I < 1000 do I := I + 1; end while;
  discard Tasks.Send(Q, 1);
end procedure;
begin scope
  const Q: channel of integer := Tasks.CreateChannel(1);
  discard go Busy(Q);
  case Tasks.ReceiveWithTimeout(Q, 10) of
    when Ok(_): panic('timer starved by CPU peer');
    when Error(_): null;
  end case;
  Time.Sleep(1000000);
  if Tasks.Receive(Q).Unwrap() <> 1 then panic('value'); end if;
end scope; end."#,
        42,
    );
    assert!(matches!(
        s.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert!(
        s.schedule_report()
            .virtual_time_millis
            .unwrap()
            .parse::<u64>()
            .unwrap()
            >= 1_000_000
    );
    assert!(
        s.schedule_report()
            .dispatches
            .unwrap()
            .parse::<u64>()
            .unwrap()
            < 100_000
    );
}

#[test]
fn settled_deadlock_is_explicit_and_retains_blocked_state() {
    let mut s = session(
        "program Blocked; uses Std.Tasks as Tasks; begin const Q: channel of integer := Tasks.CreateChannel(1); discard Tasks.Receive(Q); end.",
        0,
    );
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::ScheduleDeadlock
    );
    assert_eq!(s.schedule_report().status, "controlled");
    assert!(
        s.tasks(0, 32)
            .unwrap()
            .items
            .iter()
            .any(|task| task.state == DebugTaskState::Waiting)
    );
}

#[test]
fn unsupported_effects_reject_initializers_workers_and_nested_sync_callbacks() {
    for source in [
        "program Direct; uses Std.Time as Time; var N: integer := Time.TimestampMillis(); begin end.",
        "program Worker; uses Std.Time as Time; procedure Bad(); begin discard Time.TimestampMillis(); end procedure; begin scope discard go Bad(); end scope; end.",
        "program Nested; uses Std.Time as Time; function Inner(N: integer): integer; begin return Time.TimestampMillis(); end function; function Outer(N: integer): integer; begin return [N].Map(Inner)[0]; end function; begin discard [1].Map(Outer); end.",
    ] {
        let mut s = session(source, 42);
        let stop = stopped(s.continue_execution().unwrap());
        assert_eq!(
            stop.diagnostic.unwrap().code,
            fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT
        );
        assert_eq!(s.schedule_report().status, "rejected");
    }
}

#[test]
fn virtual_timers_selection_and_pure_callbacks_work_together() {
    let mut s = session(
        r#"program SelectTimer;
uses Std.Tasks as Tasks, Std.Time as Time;
var Seen: integer := 0;
begin
  const First := Tasks.TimerCase(1000, procedure() begin Seen := 1; end procedure);
  const Second := Tasks.TimerCase(1000, procedure() begin Seen := 2; end procedure);
  if Tasks.Select([First, Second]) <> 0 then panic('input order'); end if;
  if Seen <> 1 then panic('winner'); end if;
  if [1, 2].Map(function(N: integer): integer begin return N + 1; end function)[0] <> 2 then panic('callback'); end if;
end."#,
        7,
    );
    assert!(matches!(
        s.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn synchronous_native_callbacks_cannot_block_on_task_operations() {
    let mut s = session(
        "program InlineWait; uses Std.Time as Time; function F(N: integer): integer; begin Time.Sleep(1); return N; end function; begin discard [1].Map(F); end.",
        9,
    );
    let stop = stopped(s.continue_execution().unwrap());
    assert_eq!(
        stop.diagnostic.unwrap().code,
        fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT
    );
}

#[test]
fn inspecting_schedule_does_not_advance_execution_or_clock() {
    let s = session("program Empty; begin end.", u64::MAX);
    assert_eq!(s.schedule_report(), s.schedule_report());
    assert_eq!(
        s.schedule_report().seed.as_deref(),
        Some("18446744073709551615")
    );
    assert_eq!(
        s.schedule_report().virtual_time_millis.as_deref(),
        Some("0")
    );
}

#[test]
fn root_sleep_without_a_task_scope_uses_virtual_time() {
    let mut s = session(
        "program RootSleep; uses Std.Time as Time; begin Time.Sleep(1000000); end.",
        42,
    );
    assert!(matches!(
        s.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert!(
        s.schedule_report()
            .virtual_time_millis
            .unwrap()
            .parse::<u64>()
            .unwrap()
            >= 1_000_000
    );
}

#[test]
fn scope_and_group_cleanup_and_supervision_settle_without_false_deadlock() {
    for source in [
        include_str!("../../../../../../tests/concurrency/task_group_lifecycle_test.fpas"),
        include_str!("../../../../../../tests/concurrency/supervised_channel_selection_test.fpas"),
    ] {
        for seed in [0, 42, u64::MAX] {
            let mut s = session(source, seed);
            let result = s.continue_execution().unwrap();
            assert!(
                matches!(result, DebugRunResult::Terminated(_)),
                "{result:?}"
            );
        }
    }
}

#[test]
fn contained_child_effect_is_still_a_rejected_controlled_run() {
    let mut s = session(
        "program Contained; uses Std.Tasks as Tasks, Std.Time as Time; begin const G := Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(G, procedure(Token: Tasks.CancellationToken) begin discard Time.TimestampMillis(); end procedure); discard Tasks.CloseTaskGroup(G); end.",
        42,
    );
    let stop = stopped(s.continue_execution().unwrap());
    assert_eq!(
        stop.diagnostic.unwrap().code,
        fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT
    );
    assert_eq!(s.schedule_report().status, "rejected");
}

#[test]
fn synchronous_callback_scope_opcodes_cannot_escape_the_effect_guard() {
    let mut s = session(
        "program ScopeCallback; function F(N: integer): integer; begin scope null; end scope; return N; end function; begin discard [1].Map(F); end.",
        42,
    );
    let stop = stopped(s.continue_execution().unwrap());
    assert_eq!(
        stop.diagnostic.unwrap().code,
        fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT
    );
}

#[test]
fn virtual_clock_exhaustion_is_explicit_and_never_wraps() {
    let mut s = session(
        "program OverflowClock; var N: integer := 0; begin N := 1; end.",
        42,
    );
    s.test_advance_clock(std::time::Duration::from_millis(u64::MAX));
    assert_eq!(
        s.continue_execution().unwrap_err().kind,
        DebugErrorKind::ScheduleClockLimit
    );
    assert_eq!(
        s.schedule_report().virtual_time_millis.as_deref(),
        Some("18446744073709551615")
    );
}

#[test]
fn test_helpers_cannot_escape_filesystem_or_sealed_input_policy() {
    for body in ["discard Test.ScratchDir();", "Test.PushReadLn('injected');"] {
        let mut s = session(
            &format!("program TestEffects; uses Std.Test as Test; begin {body} end."),
            42,
        );
        let stop = stopped(s.continue_execution().unwrap());
        assert_eq!(
            stop.diagnostic.unwrap().code,
            fpas_diagnostics::codes::RUNTIME_SCHEDULE_UNSUPPORTED_EFFECT
        );
        assert_eq!(s.schedule_report().status, "rejected");
    }
}
