//! Global write and change data breakpoints.

use super::*;

const SOURCE: &str = r#"program DataBreakpoints;

mutable var Flag: integer := 0;

procedure Inner();
begin
  mutable var Nested: integer := 1;
  Nested := Nested + Flag
end;

begin
  Flag := 1;
  Flag := 1;
  Inner();
  Flag := 2
end.
"#;

fn compile_session() -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile data-breakpoint fixture"))
        .expect("debug session")
}

fn line(needle: &str) -> u32 {
    u32::try_from(
        SOURCE
            .lines()
            .position(|line| line.contains(needle))
            .expect("marker")
            + 1,
    )
    .expect("line")
}

fn stop_at(session: &mut DebugSession, needle: &str) -> u64 {
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line(needle),
            column: None,
        })
        .expect("breakpoint");
    let _ = stopped(session.continue_execution().expect("run to marker"));
    session.stack(0, 1).expect("frame").items[0].id
}

fn scope_reference(session: &mut DebugSession, scope_name: &str) -> u64 {
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    session
        .scopes(frame)
        .expect("scopes")
        .into_iter()
        .find(|scope| scope.name == scope_name)
        .expect("requested scope")
        .variables_reference
}

fn global_identity(session: &mut DebugSession) -> DebugDataLocationIdentity {
    let globals = scope_reference(session, "Globals");
    session
        .describe_data_location(globals, "Flag")
        .expect("describe Flag")
        .identity
        .expect("global identity")
}

#[test]
fn write_watch_on_global_stops_and_refreshes_inspection() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let identity = global_identity(&mut session);
    let bound = session
        .replace_data_breakpoints(vec![DataBreakpoint {
            identity,
            access: DataBreakpointAccess::Write,
        }])
        .expect("write watch");
    assert!(bound[0].verified);
    assert_eq!(bound[0].requested.identity, identity);

    let first = stopped(session.continue_execution().expect("first write"));
    assert_eq!(first.reason, DebugStopReason::DataBreakpoint);
    assert_eq!(first.breakpoint_ids, vec![bound[0].id]);
    let globals = scope_reference(&mut session, "Globals");
    let value = session
        .variables(globals, 0, 8)
        .expect("globals after write")
        .items
        .into_iter()
        .find(|variable| variable.name == "Flag")
        .expect("Flag")
        .value;
    assert_eq!(value, "1");

    let second = stopped(session.continue_execution().expect("same-value write"));
    assert_eq!(second.reason, DebugStopReason::DataBreakpoint);
    assert_eq!(second.breakpoint_ids, vec![bound[0].id]);
    assert_eq!(
        global_identity(&mut session),
        identity,
        "global identity survives the data stop"
    );
}

#[test]
fn change_watch_skips_same_value_stores() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let identity = global_identity(&mut session);
    let bound = session
        .replace_data_breakpoints(vec![
            DataBreakpoint {
                identity,
                access: DataBreakpointAccess::Write,
            },
            DataBreakpoint {
                identity,
                access: DataBreakpointAccess::Change,
            },
        ])
        .expect("write and change watches");
    let write_id = bound[0].id;
    let change_id = bound[1].id;

    let first = stopped(session.continue_execution().expect("0 to 1"));
    assert_eq!(first.reason, DebugStopReason::DataBreakpoint);
    assert_eq!(first.breakpoint_ids, vec![write_id, change_id]);

    let second = stopped(session.continue_execution().expect("same-value write"));
    assert_eq!(second.reason, DebugStopReason::DataBreakpoint);
    assert_eq!(second.breakpoint_ids, vec![write_id]);
}

#[test]
fn read_and_frame_watches_stay_unverified() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Nested := Nested + Flag");
    let identity = global_identity(&mut session);
    let locals = scope_reference(&mut session, "Locals");
    let frame = session
        .describe_data_location(locals, "Nested")
        .expect("describe nested")
        .identity
        .expect("frame identity");

    let bound = session
        .replace_data_breakpoints(vec![
            DataBreakpoint {
                identity,
                access: DataBreakpointAccess::Read,
            },
            DataBreakpoint {
                identity: frame,
                access: DataBreakpointAccess::Write,
            },
        ])
        .expect("unverified watches");
    assert!(!bound[0].verified);
    assert!(!bound[1].verified);
    assert!(
        bound[0]
            .message
            .as_deref()
            .is_some_and(|message| message.contains("Read"))
    );
    assert!(
        bound[1]
            .message
            .as_deref()
            .is_some_and(|message| message.contains("Frame-register"))
    );

    let result = session.continue_execution().expect("no data stop");
    match result {
        DebugRunResult::Terminated(_) => {}
        DebugRunResult::Stopped(stop) => {
            assert_ne!(stop.reason, DebugStopReason::DataBreakpoint);
            assert!(stop.breakpoint_ids.is_empty());
        }
    }
}

#[test]
fn oversized_data_replace_is_atomic() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let identity = global_identity(&mut session);
    let previous = session
        .replace_data_breakpoints(vec![DataBreakpoint {
            identity,
            access: DataBreakpointAccess::Write,
        }])
        .expect("initial watch");
    let limit = session.breakpoint_limits().max_breakpoints;
    let error = session
        .replace_data_breakpoints(
            (0..=limit)
                .map(|_| DataBreakpoint {
                    identity,
                    access: DataBreakpointAccess::Write,
                })
                .collect(),
        )
        .expect_err("logical breakpoint limit");
    assert_eq!(error.kind, DebugErrorKind::BreakpointLimit);

    let stop = stopped(session.continue_execution().expect("preserved watch"));
    assert_eq!(stop.reason, DebugStopReason::DataBreakpoint);
    assert_eq!(stop.breakpoint_ids, vec![previous[0].id]);
}

#[test]
fn source_breakpoint_still_stops_with_a_data_watch() {
    let mut session = compile_session();
    let _ = stop_at(&mut session, "Flag := 1;");
    let identity = global_identity(&mut session);
    let data = session
        .replace_data_breakpoints(vec![DataBreakpoint {
            identity,
            access: DataBreakpointAccess::Write,
        }])
        .expect("write watch");
    let source = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: line("Flag := 2"),
            column: None,
        })
        .expect("source breakpoint");

    let first = stopped(session.continue_execution().expect("data or source"));
    assert!(first.breakpoint_ids.contains(&data[0].id));

    let mut ids = Vec::new();
    loop {
        let stop = stopped(session.continue_execution().expect("reach source"));
        ids.extend(stop.breakpoint_ids.iter().copied());
        if ids.contains(&source.id) {
            break;
        }
        if stop.reason != DebugStopReason::DataBreakpoint {
            panic!("expected to reach the source breakpoint: {stop:?}");
        }
    }
    assert!(ids.contains(&source.id));
}
