//! Controlled-call integration contracts shared by JSONL and DAP adapters.

#![allow(
    clippy::expect_used,
    reason = "integration fixtures keep compiler and debugger failures local"
)]

use std::thread;
use std::time::Duration;

use fpas_vm::{
    DebugErrorKind, DebugEvaluationLimits, DebugExpression, DebugRunResult, DebugSession,
    SourceBreakpoint,
};

fn compile(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    fpas_compiler::compile(&program).expect("compile controlled-call fixture")
}

fn call(name: &str, arguments: Vec<DebugExpression>) -> DebugExpression {
    DebugExpression::Call {
        callee: Box::new(DebugExpression::Name(name.to_string())),
        arguments,
    }
}

#[test]
fn methods_properties_static_constructors_records_and_intrinsics_execute() {
    let source = "\
program DebugMembers;
uses Std.Math;
type
  Counter = record
    Value: integer;
    static function Create(Value: integer): Counter;
    begin
      return record Value := Value; end
    end;
    function Double(Self: Counter): integer;
    begin
      return Self.Value * 2
    end;
    function ReadNumber(Self: Counter): integer;
    begin
      return Self.Value
    end;
    property Number: integer read ReadNumber;
  end;
procedure Touch();
begin
end;
begin
end.";
    let mut server = fpas_debug::jsonl::JsonlServer::new(fpas_debug::PreparedDebugTarget::new(
        compile(source),
        Vec::new(),
    ))
    .expect("JSONL server");
    let request = |id, expression: &str| {
        serde_json::json!({"type":"request","id":id,"command":"evaluate","arguments":{"expression":expression}}).to_string()
    };
    let _ = server.handle_line(
        &serde_json::json!({"type":"request","id":1,"command":"initialize","arguments":{"version":2}}).to_string(),
    );
    let _ = server.handle_line(
        &serde_json::json!({"type":"request","id":2,"command":"launch","arguments":{"stop_on_entry":true}}).to_string(),
    );

    let cases = [
        ("Counter.Create(6).Double()", "12"),
        ("Counter.Create(7).Number", "7"),
        ("(record Value := 8; end).Double()", "16"),
        ("Std.Math.Abs(-9)", "9"),
        ("Touch()", "()"),
        ("try Some(11)", "11"),
        ("[1, 2, 3][1]", "2"),
    ];
    for (index, (expression, expected)) in cases.into_iter().enumerate() {
        let records = server.handle_line(&request(10 + index as u64, expression));
        assert_eq!(
            records[0]["body"]["result"], expected,
            "{expression}: {records:?}"
        );
    }
}

#[test]
fn denied_host_effects_never_reach_the_live_console() {
    let source = "\
program DebugDenied;
uses Std.Console;
function Noisy(): integer;
begin
  Std.Console.WriteLn('leak');
  return 1
end;
begin
end.";
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    let before = session.output();

    let failure = session
        .evaluate(&call("Noisy", Vec::new()), None)
        .expect_err("host output must be denied");

    assert_eq!(failure.kind, DebugErrorKind::ForbiddenCallEffect);
    assert_eq!(session.output().lines, before.lines);

    let direct = session
        .evaluate(
            &call(
                "WriteLn",
                vec![DebugExpression::String("also hidden".to_string())],
            ),
            None,
        )
        .expect_err("imported short intrinsic name must reach the same policy");
    assert_eq!(direct.kind, DebugErrorKind::ForbiddenCallEffect);
    assert_eq!(session.output().lines, before.lines);
}

#[test]
fn detached_global_writes_roll_back_and_stop_identity_survives() {
    let source = "\
program DebugRollback;
mutable var Counter: integer := 5;
function Increment(): integer;
begin
  Counter := Counter + 1;
  return Counter
end;
begin
  mutable var Anchor: integer := Counter;
  Anchor := Anchor + 1
end.";
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: 10,
            column: None,
        })
        .expect("set breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    let DebugRunResult::Stopped(stop) = session.continue_execution().expect("reach breakpoint")
    else {
        panic!("expected stable stop after global initialization");
    };
    let before = stop.clone();

    let result = session
        .evaluate(&call("Increment", Vec::new()), None)
        .expect("detached write call");
    let live = session
        .evaluate(&DebugExpression::Name("Counter".to_string()), None)
        .expect("live global after call");

    assert_eq!(result.value, "6");
    assert_eq!(live.value, "5");
    assert_eq!(session.last_stop(), &before);
}

#[test]
fn visible_first_class_closure_uses_detached_mutable_captures() {
    let source = "\
program DebugClosure;
begin
  mutable var Base: integer := 10;
  var AddBase: function(Value: integer): integer :=
    function(Value: integer): integer
    begin
      return Base + Value
    end;
  mutable var Marker: integer := 0;
  Marker := Marker + 1
end.";
    let mut session = DebugSession::new(compile(source)).expect("debug session");
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: 10,
            column: None,
        })
        .expect("set closure breakpoint");
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session.continue_execution().expect("reach closure stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;

    let result = session
        .evaluate(
            &call("AddBase", vec![DebugExpression::Integer(7)]),
            Some(frame),
        )
        .expect("detached captured closure");

    assert_eq!(result.value, "17");
}

#[test]
fn timeout_and_cooperative_cancellation_leave_the_session_stopped() {
    let source = "\
program DebugLimits;
function Forever(): integer;
begin
  while true do begin end;
  return 0
end;
begin
end.";
    let mut timed = DebugSession::new(compile(source)).expect("timed session");
    let before = timed.last_stop().clone();
    let failure = timed
        .evaluate_with_limits(
            &call("Forever", Vec::new()),
            None,
            DebugEvaluationLimits {
                call_timeout: Duration::ZERO,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("zero deadline");
    assert_eq!(failure.kind, DebugErrorKind::CallTimeout);
    assert_eq!(timed.last_stop(), &before);

    let mut cancelled = DebugSession::new(compile(source)).expect("cancel session");
    let handle = cancelled.evaluation_cancel_handle();
    let cancellation = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        handle.cancel();
    });
    let failure = cancelled
        .evaluate_with_limits(
            &call("Forever", Vec::new()),
            None,
            DebugEvaluationLimits {
                call_timeout: Duration::from_secs(1),
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("cooperative cancellation");
    cancellation.join().expect("cancellation thread");
    assert_eq!(failure.kind, DebugErrorKind::CallCancelled);
}
