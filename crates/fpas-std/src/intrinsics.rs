//! Dispatches intrinsic instructions to their standard-library unit modules.
//! Console, task wait, and higher-order (callback) intrinsics are handled in `fpas-vm`, not here.
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root).
//! **Maintenance:** Runtime ownership is declared by `fpas-bytecode::Intrinsic::owner`; this module
//! directly routes standard-owned families to exactly one implementation module.

use crate::array;
use crate::conv;
use crate::dict;
use crate::env;
use crate::error::{StdError, std_internal_error, std_runtime_error};
use crate::fs;
use crate::intrinsic_args::IntrinsicCall;
use crate::json;
use crate::math;
use crate::parse;
use crate::path;
use crate::proc;
use crate::random;
use crate::result_option;
use crate::str;
use crate::time;
use crate::toml;
use fpas_bytecode::{Intrinsic, IntrinsicOwner, SourceLocation, Value};

fn dispatch_intrinsic(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<(), StdError> {
    let owner = intrinsic.owner();
    if owner != IntrinsicOwner::Standard {
        return Err(std_internal_error(
            format!(
                "internal: {} is owned by the VM ({owner:?})",
                intrinsic.debugger_name()
            ),
            "This indicates a VM dispatch bug. Route the intrinsic through its declared runtime owner.",
            location,
        ));
    }

    let handled = match intrinsic {
        Intrinsic::Str(_) => str::run(intrinsic, call, location),
        Intrinsic::Conv(_) => conv::run(intrinsic, call, location),
        Intrinsic::Parse(_) => parse::run(intrinsic, call, location),
        Intrinsic::Math(_) => math::run(intrinsic, call, location),
        Intrinsic::Random(_) => random::run(intrinsic, call, location),
        Intrinsic::Array(_) => array::run(intrinsic, call, location),
        Intrinsic::Dict(_) => dict::run(intrinsic, call, location),
        Intrinsic::Env(_) => env::run(intrinsic, call, location),
        Intrinsic::Path(_) => path::run(intrinsic, call, location),
        Intrinsic::Proc(_) => proc::run(intrinsic, call, location),
        Intrinsic::Fs(_) => fs::run(intrinsic, call, location),
        Intrinsic::Json(_) => json::run(intrinsic, call, location),
        Intrinsic::Result(_) | Intrinsic::Option(_) => {
            result_option::run(intrinsic, call, location)
        }
        Intrinsic::Time(_) => time::run(intrinsic, call, location),
        Intrinsic::Toml(_) => toml::run(intrinsic, call, location),
        Intrinsic::Test(_) => crate::test::run(intrinsic, call, location),
        Intrinsic::Args(_)
        | Intrinsic::Console(_)
        | Intrinsic::Net(_)
        | Intrinsic::Http(_)
        | Intrinsic::Task(_) => {
            return Err(std_internal_error(
                format!(
                    "internal: {} has inconsistent runtime ownership",
                    intrinsic.debugger_name()
                ),
                "This indicates a bytecode intrinsic-classification bug.",
                location,
            ));
        }
    }?;

    if handled.is_some() {
        Ok(())
    } else {
        Err(std_internal_error(
            format!("unknown or unimplemented intrinsic reached std dispatch ({intrinsic:?})"),
            "This indicates a VM dispatch bug: every standard intrinsic must be implemented by its owning fpas-std module.",
            location,
        ))
    }
}

/// Execute a non-hosted standard-library intrinsic from a borrowed register argument window.
///
/// The returned value is `None` for procedures. The argument slice is never mutated; aggregate
/// values retain copy-on-write value semantics when an implementation needs ownership.
///
/// # Errors
///
/// Returns a structured standard-runtime diagnostic for an unsupported hosted intrinsic, an
/// argument count/type mismatch, or a unit-specific runtime failure.
pub fn run_intrinsic_borrowed(
    intrinsic: Intrinsic,
    arguments: &[Value],
    location: SourceLocation,
    factory: &dyn crate::AggregateFactory,
) -> Result<Option<Value>, StdError> {
    let mut call = IntrinsicCall::new(arguments, factory);
    dispatch_intrinsic(intrinsic, &mut call, location)?;
    let (consumed, result) = call.finish();
    if consumed != arguments.len() {
        return Err(std_runtime_error(
            fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Intrinsic argument count mismatch: decoded {consumed}, received {}",
                arguments.len()
            ),
            "Check the compiler intrinsic signature and register argument count.",
            location,
        ));
    }
    Ok(result)
}

#[cfg(test)]
pub(crate) static TEST_AGGREGATES: TestAggregateFactory = TestAggregateFactory;

#[cfg(test)]
pub(crate) struct TestAggregateFactory;

#[cfg(test)]
impl crate::AggregateFactory for TestAggregateFactory {
    fn record(
        &self,
        type_name: &str,
        values: Vec<Value>,
        _location: SourceLocation,
    ) -> Result<Value, StdError> {
        let fields = match type_name {
            crate::std_symbols::STD_PROC_PROCESS_OUTPUT => {
                ["ExitCode", "Stdout", "Stderr"].map(str::to_owned).to_vec()
            }
            _ => (0..values.len())
                .map(|index| format!("field{index}"))
                .collect(),
        };
        Ok(Value::Record(fpas_bytecode::SharedRecord::new(
            std::sync::Arc::new(fpas_bytecode::RuntimeRecordLayout {
                record: fpas_bytecode::RecordTypeId::new(test_id(type_name)),
                type_name: type_name.to_owned(),
                fields,
            }),
            values,
        )))
    }

    fn enumeration(
        &self,
        type_name: &str,
        variant: &str,
        values: Vec<Value>,
        _location: SourceLocation,
    ) -> Result<Value, StdError> {
        Ok(Value::Enum(fpas_bytecode::SharedEnum::new(
            std::sync::Arc::new(fpas_bytecode::RuntimeEnumLayout {
                enumeration: fpas_bytecode::EnumTypeId::new(test_id(type_name)),
                variant_id: fpas_bytecode::EnumVariantId::new(test_id(&format!(
                    "{type_name}.{variant}"
                ))),
                type_name: type_name.to_owned(),
                variant: variant.to_owned(),
                fields: (0..values.len())
                    .map(|index| format!("field{index}"))
                    .collect(),
            }),
            values,
        )))
    }
}

#[cfg(test)]
fn test_id(name: &str) -> u16 {
    name.bytes().fold(0_u16, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(u16::from(byte))
    })
}

#[cfg(test)]
pub(crate) fn execute_test_intrinsic(
    intrinsic: Intrinsic,
    arguments: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<(), StdError> {
    let result = run_intrinsic_borrowed(intrinsic, arguments, location, &TEST_AGGREGATES)?;
    arguments.clear();
    if let Some(value) = result {
        arguments.push(value);
    }
    Ok(())
}

#[cfg(test)]
mod vm_only_guard_tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{TEST_AGGREGATES, execute_test_intrinsic, run_intrinsic_borrowed};
    use fpas_bytecode::{
        ArgsIntrinsic, ArrayIntrinsic, ConsoleIntrinsic, Intrinsic, SourceLocation, StrIntrinsic,
        TaskIntrinsic, Value,
    };

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn args_param_count_is_vm_only() {
        let err = execute_test_intrinsic(
            Intrinsic::Args(ArgsIntrinsic::ParamCount),
            &mut Vec::new(),
            loc(),
        )
        .expect_err("expected internal error");
        assert!(err.message.contains("Std.Args"), "message={}", err.message);
    }

    #[test]
    fn console_poll_event_is_vm_only() {
        let err = execute_test_intrinsic(
            Intrinsic::Console(ConsoleIntrinsic::PollEvent),
            &mut Vec::new(),
            loc(),
        )
        .expect_err("expected internal error");
        assert!(
            err.message.contains("Std.Console.PollEvent")
                && err.message.contains("owned by the VM"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn task_wait_is_vm_only() {
        let err =
            execute_test_intrinsic(Intrinsic::Task(TaskIntrinsic::Wait), &mut Vec::new(), loc())
                .expect_err("err");
        assert!(
            err.message.contains("Std.Task.Wait") && err.message.contains("owned by the VM"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn array_map_is_vm_only() {
        let err = execute_test_intrinsic(
            Intrinsic::Array(ArrayIntrinsic::Map),
            &mut Vec::new(),
            loc(),
        )
        .expect_err("err");
        assert!(
            err.message.contains("Std.Array.Map") && err.message.contains("owned by the VM"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn str_length_still_dispatches() {
        let mut stack = vec![Value::Str("ab".into())];
        execute_test_intrinsic(Intrinsic::Str(StrIntrinsic::Length), &mut stack, loc()).unwrap();
        assert_eq!(stack, vec![Value::Integer(2)]);
    }

    #[test]
    fn borrowed_arguments_are_not_consumed_or_mutated() {
        let arguments = vec![Value::Array(
            vec![Value::Integer(1), Value::Integer(2)].into(),
        )];
        let before = arguments.clone();
        let result = run_intrinsic_borrowed(
            Intrinsic::Array(ArrayIntrinsic::Reverse),
            &arguments,
            loc(),
            &TEST_AGGREGATES,
        )
        .unwrap();
        assert_eq!(arguments, before);
        assert_eq!(
            result,
            Some(Value::Array(
                vec![Value::Integer(2), Value::Integer(1)].into()
            ))
        );
    }

    #[test]
    fn borrowed_dispatch_rejects_extra_and_wrong_typed_arguments() {
        let extra = run_intrinsic_borrowed(
            Intrinsic::Str(StrIntrinsic::Length),
            &[Value::Integer(99), Value::Str("text".into())],
            loc(),
            &TEST_AGGREGATES,
        )
        .expect_err("extra argument");
        assert!(extra.message.contains("argument count mismatch"));

        let wrong = run_intrinsic_borrowed(
            Intrinsic::Str(StrIntrinsic::Length),
            &[Value::Integer(99)],
            loc(),
            &TEST_AGGREGATES,
        )
        .expect_err("wrong type");
        assert!(wrong.message.contains("Expected string"));
    }
}
