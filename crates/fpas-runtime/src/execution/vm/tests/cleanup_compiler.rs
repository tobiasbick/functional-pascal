//! Cancellation at compiler-generated resource publication boundaries.

use crate::execution::vm::{dispatch::DispatchStep, tasks::TaskScheduler, worker::Worker};
use std::sync::Arc;

#[test]
fn cancellation_after_wrapped_and_branching_factory_calls_releases_the_resource() {
    for expression in [
        "Acquire().Unwrap()",
        "try Acquire().MapError(Convert)",
        "if true then Acquire().Unwrap() else Acquire().Unwrap() end if",
        "if false then Acquire().Unwrap() else Acquire().Unwrap() end if",
        "case true of when true: Acquire().Unwrap(); when false: Acquire().Unwrap(); end case",
        "case false of when true: Acquire().Unwrap(); when false: Acquire().Unwrap(); end case",
    ] {
        let source = format!(
            r#"program T;
type R = record
  release procedure Close(Self: R); begin panic('released'); end procedure;
end record;
function Acquire(): result of (R, string); begin return Ok(R()); end function;
function Convert(Message: string): string; begin return Message; end function;
function Work(): result of (integer, string);
begin
  using const Item := {expression};
  return Ok(1);
end function;
begin discard Work(); end."#
        );
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{expression}: {errors:?}");
        let image = fpas_compiler::compile(&program).expect("compile fixture");
        let executable = image.executable();
        let factory = executable
            .functions
            .iter()
            .position(|function| {
                executable.strings.get(function.name).is_some_and(|name| {
                    name.eq_ignore_ascii_case("T.Acquire") || name.eq_ignore_ascii_case("Acquire")
                })
            })
            .expect("factory");
        let return_addresses: Vec<_> = executable
            .code
            .iter()
            .enumerate()
            .filter(|(_, instruction)| {
                instruction.opcode() == Ok(crate::Opcode::CallDirect)
                    && instruction
                        .abc_operands()
                        .is_ok_and(|operands| usize::from(operands.b) == factory)
            })
            .map(|(index, _)| index + 1)
            .collect();
        assert!(!return_addresses.is_empty(), "{expression}");
        let scheduler = Arc::new(TaskScheduler::new());
        let (scope, _) = scheduler.scopes.open();
        let mut worker = Worker::new(Arc::new(image))
            .expect("worker")
            .with_scheduler(Some(scheduler.clone()));
        worker.task_id = 1;
        worker.owner_scope = scheduler.scopes.enroll(scope, 1);
        let mut returned = false;
        for _ in 0..1000 {
            if return_addresses.contains(&worker.ip) {
                returned = true;
                break;
            }
            assert!(
                matches!(
                    worker.dispatch_one().expect("acquire"),
                    DispatchStep::Continue
                ),
                "{expression}"
            );
        }
        assert!(returned, "factory did not return: {expression}");
        scheduler.scopes.cancel(scope);
        let error = loop {
            match worker.dispatch_one() {
                Err(error) => break error,
                Ok(DispatchStep::Continue) => {}
                Ok(_) => panic!("cancellation must fail the task: {expression}"),
            }
        };
        assert_eq!(
            error.code,
            fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC,
            "{expression}"
        );
        assert!(error.message.contains("released"), "{expression}");
        let diagnostics = worker.hosted.supplemental_diagnostics.lock().unwrap();
        assert!(diagnostics.is_empty(), "{expression}");
    }
}
