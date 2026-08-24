//! Dispatches intrinsic instructions to their standard-library unit modules.
//! Console, task wait, and higher-order (callback) intrinsics are handled in `fpas-vm`, not here.
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root).
//! **Maintenance:** When adding or rerouting intrinsics, update the README, the relevant unit `.md` file,
//! `fpas-bytecode::Intrinsic`, and the VM-only `matches!` guards below (mirror `try_exec_*` in `fpas-vm`).

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
use fpas_bytecode::{
    ArgsIntrinsic, ArrayIntrinsic, ConsoleIntrinsic, DictIntrinsic, Intrinsic, NetIntrinsic,
    OptionIntrinsic, ResultIntrinsic, SourceLocation, TaskIntrinsic, Value,
};

type StdUnitDispatch =
    fn(Intrinsic, &mut IntrinsicCall<'_>, SourceLocation) -> Result<Option<()>, StdError>;

const STD_UNIT_DISPATCHERS: &[StdUnitDispatch] = &[
    env::run,
    path::run,
    proc::run,
    fs::run,
    str::run,
    conv::run,
    parse::run,
    math::run,
    random::run,
    array::run,
    result_option::run,
    dict::run,
    json::run,
    time::run,
    toml::run,
    crate::test::run,
];

fn dispatch_intrinsic(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<(), StdError> {
    if matches!(
        intrinsic,
        Intrinsic::Args(ArgsIntrinsic::ParamCount)
            | Intrinsic::Args(ArgsIntrinsic::ParamStr)
            | Intrinsic::Console(ConsoleIntrinsic::ReadLn)
            | Intrinsic::Console(ConsoleIntrinsic::Read)
            | Intrinsic::Console(ConsoleIntrinsic::ReadKey)
            | Intrinsic::Console(ConsoleIntrinsic::KeyPressed)
            | Intrinsic::Console(ConsoleIntrinsic::ReadKeyEvent)
            | Intrinsic::Console(ConsoleIntrinsic::ClrScr)
            | Intrinsic::Console(ConsoleIntrinsic::ClrEol)
            | Intrinsic::Console(ConsoleIntrinsic::GotoXY)
            | Intrinsic::Console(ConsoleIntrinsic::WhereX)
            | Intrinsic::Console(ConsoleIntrinsic::WhereY)
            | Intrinsic::Console(ConsoleIntrinsic::WindMin)
            | Intrinsic::Console(ConsoleIntrinsic::WindMax)
            | Intrinsic::Console(ConsoleIntrinsic::DelLine)
            | Intrinsic::Console(ConsoleIntrinsic::InsLine)
            | Intrinsic::Console(ConsoleIntrinsic::Window)
            | Intrinsic::Console(ConsoleIntrinsic::TextColor)
            | Intrinsic::Console(ConsoleIntrinsic::TextBackground)
            | Intrinsic::Console(ConsoleIntrinsic::HighVideo)
            | Intrinsic::Console(ConsoleIntrinsic::LowVideo)
            | Intrinsic::Console(ConsoleIntrinsic::NormVideo)
            | Intrinsic::Console(ConsoleIntrinsic::TextAttr)
            | Intrinsic::Console(ConsoleIntrinsic::SetTextAttr)
            | Intrinsic::Console(ConsoleIntrinsic::Delay)
            | Intrinsic::Console(ConsoleIntrinsic::CursorOn)
            | Intrinsic::Console(ConsoleIntrinsic::CursorBig)
            | Intrinsic::Console(ConsoleIntrinsic::CursorOff)
            | Intrinsic::Console(ConsoleIntrinsic::TextMode)
            | Intrinsic::Console(ConsoleIntrinsic::LastMode)
            | Intrinsic::Console(ConsoleIntrinsic::ScreenWidth)
            | Intrinsic::Console(ConsoleIntrinsic::ScreenHeight)
            | Intrinsic::Console(ConsoleIntrinsic::Sound)
            | Intrinsic::Console(ConsoleIntrinsic::NoSound)
            | Intrinsic::Console(ConsoleIntrinsic::AssignCrt)
            | Intrinsic::Console(ConsoleIntrinsic::EventPending)
            | Intrinsic::Console(ConsoleIntrinsic::ReadEvent)
            | Intrinsic::Console(ConsoleIntrinsic::EnableRawMode)
            | Intrinsic::Console(ConsoleIntrinsic::DisableRawMode)
            | Intrinsic::Console(ConsoleIntrinsic::EnterAltScreen)
            | Intrinsic::Console(ConsoleIntrinsic::LeaveAltScreen)
            | Intrinsic::Console(ConsoleIntrinsic::EnableMouse)
            | Intrinsic::Console(ConsoleIntrinsic::DisableMouse)
            | Intrinsic::Console(ConsoleIntrinsic::EnableFocus)
            | Intrinsic::Console(ConsoleIntrinsic::DisableFocus)
            | Intrinsic::Console(ConsoleIntrinsic::EnablePaste)
            | Intrinsic::Console(ConsoleIntrinsic::DisablePaste)
            | Intrinsic::Console(ConsoleIntrinsic::AcquireInteractiveTerminal)
            | Intrinsic::Console(ConsoleIntrinsic::ReleaseInteractiveTerminal)
            | Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout)
            | Intrinsic::Console(ConsoleIntrinsic::PollEvent)
            | Intrinsic::Console(ConsoleIntrinsic::TextColorRGB)
            | Intrinsic::Console(ConsoleIntrinsic::TextBackgroundRGB)
            | Intrinsic::Console(ConsoleIntrinsic::TextColor256)
            | Intrinsic::Console(ConsoleIntrinsic::TextBackground256)
            | Intrinsic::Console(ConsoleIntrinsic::CrtColor)
            | Intrinsic::Console(ConsoleIntrinsic::Ansi256Color)
            | Intrinsic::Console(ConsoleIntrinsic::RgbColor)
            | Intrinsic::Console(ConsoleIntrinsic::BeginFrame)
            | Intrinsic::Console(ConsoleIntrinsic::Present)
            | Intrinsic::Console(ConsoleIntrinsic::PutCell)
            | Intrinsic::Console(ConsoleIntrinsic::GetCell)
            | Intrinsic::Console(ConsoleIntrinsic::FillRect)
            | Intrinsic::Console(ConsoleIntrinsic::WriteCells)
            | Intrinsic::Console(ConsoleIntrinsic::SaveRegion)
            | Intrinsic::Console(ConsoleIntrinsic::RestoreRegion)
            | Intrinsic::Console(ConsoleIntrinsic::DiscardRegion)
            | Intrinsic::Console(ConsoleIntrinsic::DisplayWidth)
            | Intrinsic::Console(ConsoleIntrinsic::GraphemeWidth)
            | Intrinsic::Console(ConsoleIntrinsic::SplitGraphemes)
            | Intrinsic::Console(ConsoleIntrinsic::Write)
            | Intrinsic::Console(ConsoleIntrinsic::WriteLn)
    ) {
        return Err(std_internal_error(
            "internal: Std.Args and Std.Console intrinsics are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    if matches!(
        intrinsic,
        Intrinsic::Net(
            NetIntrinsic::Connect
                | NetIntrinsic::ConnectTls
                | NetIntrinsic::SetTimeout
                | NetIntrinsic::Read
                | NetIntrinsic::Write
                | NetIntrinsic::Close
        )
    ) {
        return Err(std_internal_error(
            "internal: Std.Net intrinsics are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    if matches!(
        intrinsic,
        Intrinsic::Task(TaskIntrinsic::Wait) | Intrinsic::Task(TaskIntrinsic::WaitAll)
    ) {
        return Err(std_internal_error(
            "internal: Std.Task wait intrinsics (Wait, WaitAll) are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    if matches!(
        intrinsic,
        Intrinsic::Array(ArrayIntrinsic::Map)
            | Intrinsic::Array(ArrayIntrinsic::Filter)
            | Intrinsic::Array(ArrayIntrinsic::Reduce)
            | Intrinsic::Array(ArrayIntrinsic::Find)
            | Intrinsic::Array(ArrayIntrinsic::FindIndex)
            | Intrinsic::Array(ArrayIntrinsic::Any)
            | Intrinsic::Array(ArrayIntrinsic::All)
            | Intrinsic::Array(ArrayIntrinsic::FlatMap)
            | Intrinsic::Array(ArrayIntrinsic::ForEach)
            | Intrinsic::Result(ResultIntrinsic::Map)
            | Intrinsic::Result(ResultIntrinsic::AndThen)
            | Intrinsic::Result(ResultIntrinsic::OrElse)
            | Intrinsic::Option(OptionIntrinsic::Map)
            | Intrinsic::Option(OptionIntrinsic::AndThen)
            | Intrinsic::Option(OptionIntrinsic::OrElse)
            | Intrinsic::Dict(DictIntrinsic::Map)
            | Intrinsic::Dict(DictIntrinsic::Filter)
    ) {
        return Err(std_internal_error(
            "internal: higher-order Std intrinsics (function callbacks) are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    for dispatch in STD_UNIT_DISPATCHERS {
        if dispatch(intrinsic, call, location)?.is_some() {
            return Ok(());
        }
    }

    Err(std_internal_error(
        format!("unknown or unimplemented intrinsic reached std dispatch ({intrinsic:?})"),
        "This indicates a VM dispatch bug: console, TUI, task wait, and callback-based std opcodes must be handled in the VM; all other std opcodes must be implemented in fpas-std. Please report this as a compiler/runtime issue.",
        location,
    ))
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
            err.message.contains("Std.Args and Std.Console"),
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
            err.message.contains("Std.Task wait"),
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
            err.message.contains("higher-order Std intrinsics"),
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
