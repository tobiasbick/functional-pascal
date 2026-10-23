//! Live-session cache stability and teardown after retained inspection/export roots.
use crate::measurement::{completed_threads, steady_state};
use crate::workloads::{DECLARATIONS, function, image, schedule};
use fpas_bytecode::Value;
use fpas_vm::{CallbackSession, DebugExpression, DebugRunResult, DebugSession, SourceBreakpoint};
use std::sync::Arc;

pub fn callbacks() {
    let image = Arc::new(image(&format!(
        "program Callbacks; {DECLARATIONS}
        procedure Fail(); begin const Garbage := Make();
          if Garbage() <> 32897 then panic('garbage payload'); end if;
          panic('expected callback failure');
        end procedure;
        begin end."
    )));
    let make = function(&image, "Make");
    let invoke = function(&image, "Invoke");
    let fail = function(&image, "Fail");
    completed_threads("callback session teardown", 12, || {
        let mut session =
            CallbackSession::from_shared(image.clone()).with_collection_schedule(schedule());
        let keep = session.invoke(make, vec![]).unwrap().value;
        steady_state("callback live session", 32, || {
            let exported = session.invoke(make, vec![]).unwrap().value;
            let alias = exported.clone();
            assert_eq!(
                session
                    .invoke(invoke, vec![exported.into()])
                    .unwrap()
                    .value
                    .try_to_data()
                    .unwrap(),
                Value::Integer(32897)
            );
            assert_eq!(
                session
                    .invoke(invoke, vec![alias.into()])
                    .unwrap()
                    .value
                    .try_to_data()
                    .unwrap(),
                Value::Integer(32898)
            );
            assert!(
                session
                    .invoke(fail, vec![])
                    .unwrap_err()
                    .message
                    .contains("expected callback failure")
            );
        });
        assert_eq!(
            session
                .invoke(invoke, vec![keep.clone().into()])
                .unwrap()
                .value
                .try_to_data()
                .unwrap(),
            Value::Integer(32897)
        );
        drop(session);
        assert_eq!(keep.type_name(), "function");
        drop(keep);
    });
}

pub fn debugger() {
    let source = format!(
        r#"program DebugMemory;
{DECLARATIONS}
function Inspect(Next: function(): integer): integer;
begin return 42; end function;
procedure Tick();
begin
  const Current := Make();
  const Check := Current();
  if Check <> 32897 then panic('debug state'); end if;
end procedure;
begin
  for I: integer := 1 to 512 do Tick(); end for;
end."#
    );
    let image = image(&source);
    completed_threads("debugger session teardown", 2, || {
        let mut session = DebugSession::new(image.clone())
            .unwrap()
            .with_collection_schedule(schedule());
        let line = u32::try_from(
            source
                .lines()
                .position(|line| line.contains("const Check :="))
                .unwrap()
                + 1,
        )
        .unwrap();
        assert!(
            session
                .set_breakpoint(SourceBreakpoint {
                    source: "<memory>".into(),
                    line,
                    column: None,
                })
                .unwrap()
                .instruction
                .is_some()
        );
        steady_state("debugger live session", 32, || {
            assert!(matches!(
                session.continue_execution().unwrap(),
                DebugRunResult::Stopped(_)
            ));
            inspect_current(&mut session);
            // Resume past Tick's scope exit: temporary inspection/evaluation roots
            // and their randomly sized snapshot maps must be gone at this baseline.
            assert!(matches!(
                session.step_out().unwrap(),
                DebugRunResult::Stopped(_)
            ));
            assert!(
                !session.stack(0, 1).unwrap().items[0]
                    .name
                    .eq_ignore_ascii_case("Tick")
            );
        });
        assert!(matches!(
            session.continue_execution().unwrap(),
            DebugRunResult::Stopped(_)
        ));
        inspect_current(&mut session);
        session.disconnect();
    });
}

fn inspect_current(session: &mut DebugSession) {
    let frames = session.stack(0, 1).unwrap();
    assert!(
        frames.items[0].name.eq_ignore_ascii_case("Tick"),
        "{frames:?}"
    );
    let frame = frames.items[0].id;
    for scope in session.scopes(frame).unwrap() {
        let variables = session.variables(scope.variables_reference, 0, 16).unwrap();
        for variable in variables.items {
            if variable.variables_reference != 0 {
                drop(
                    session
                        .variables(variable.variables_reference, 0, 4)
                        .unwrap(),
                );
            }
        }
    }
    drop(
        session
            .evaluate(&DebugExpression::Name("Current".into()), Some(frame))
            .unwrap_or_else(|error| panic!("{error:?}; {frames:?}; {:?}", session.scopes(frame))),
    );
    let evaluated = session
        .evaluate(
            &DebugExpression::Call {
                callee: Box::new(DebugExpression::Callable("Inspect".into())),
                arguments: vec![DebugExpression::Name("Current".into())],
            },
            Some(frame),
        )
        .unwrap();
    assert_eq!(evaluated.value, "42");
}
