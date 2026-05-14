//! Dispatches `Op::Intrinsic` to unit modules (`str`, `conv`, `math`, `array`, `result_option`, `dict`).
//! Console, TUI, task wait, and higher-order (callback) intrinsics are handled in `fpas-vm`, not here.
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root).
//! **Maintenance:** When adding or rerouting intrinsics, update the README, the relevant unit `.md` file,
//! `fpas-bytecode::Intrinsic`, and the VM-only `matches!` guards below (mirror `try_exec_*` in `fpas-vm`).

use crate::array;
use crate::conv;
use crate::dict;
use crate::error::{StdError, std_internal_error};
use crate::math;
use crate::result_option;
use crate::str;
use fpas_bytecode::{ArrayIntrinsic, ConsoleIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation, TaskIntrinsic, TuiIntrinsic, Value};
/// Execute a standard-library intrinsic; mutates `stack` (Pascal call order: args already pushed).
pub fn run_intrinsic(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<(), StdError> {
    if matches!(
        intrinsic,
        Intrinsic::Console(ConsoleIntrinsic::ReadLn)
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
            | Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout)
            | Intrinsic::Console(ConsoleIntrinsic::PollEvent)
            | Intrinsic::Console(ConsoleIntrinsic::TextColorRGB)
            | Intrinsic::Console(ConsoleIntrinsic::TextBackgroundRGB)
            | Intrinsic::Console(ConsoleIntrinsic::TextColor256)
            | Intrinsic::Console(ConsoleIntrinsic::TextBackground256)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationOpen)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationClose)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationSize)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationReadEvent)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationReadEventTimeout)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationPollEvent)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationRedrawPending)
            | Intrinsic::Tui(TuiIntrinsic::ApplicationConfigure)
            | Intrinsic::Tui(TuiIntrinsic::HostPollNext)
            | Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed)
            | Intrinsic::Tui(TuiIntrinsic::HostInvokeOnKeyPressed)
            | Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize)
            | Intrinsic::Tui(TuiIntrinsic::HostProcessNext)
            | Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint)
            | Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle)
            | Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw)
            | Intrinsic::Tui(TuiIntrinsic::HostRunLoop)
            | Intrinsic::Tui(TuiIntrinsic::HostRequestQuit)
            | Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit)
    ) {
        return Err(std_internal_error(
            "internal: Std.Console and Std.Tui intrinsics are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    if matches!(intrinsic, Intrinsic::Task(TaskIntrinsic::Wait) | Intrinsic::Task(TaskIntrinsic::WaitAll)) {
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

    if str::run(intrinsic, stack, location)?.is_some() {
        return Ok(());
    }
    if conv::run(intrinsic, stack, location)?.is_some() {
        return Ok(());
    }
    if math::run(intrinsic, stack, location)?.is_some() {
        return Ok(());
    }
    if array::run(intrinsic, stack, location)?.is_some() {
        return Ok(());
    }
    if result_option::run(intrinsic, stack, location)?.is_some() {
        return Ok(());
    }
    if dict::run(intrinsic, stack, location)?.is_some() {
        return Ok(());
    }

    Err(std_internal_error(
        format!("unknown or unimplemented intrinsic reached std dispatch ({intrinsic:?})"),
        "This indicates a VM dispatch bug: console, TUI, task wait, and callback-based std opcodes must be handled in the VM; all other std opcodes must be implemented in fpas-std. Please report this as a compiler/runtime issue.",
        location,
    ))
}

#[cfg(test)]
mod vm_only_guard_tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::run_intrinsic;
    use fpas_bytecode::{
        ArrayIntrinsic, ConsoleIntrinsic, Intrinsic, SourceLocation, StrIntrinsic,
        TaskIntrinsic, Value,
    };

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn console_poll_event_is_vm_only() {
        let err = run_intrinsic(Intrinsic::Console(ConsoleIntrinsic::PollEvent), &mut Vec::new(), loc())
            .expect_err("expected internal error");
        assert!(
            err.message.contains("Std.Console and Std.Tui"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn task_wait_is_vm_only() {
        let err = run_intrinsic(Intrinsic::Task(TaskIntrinsic::Wait), &mut Vec::new(), loc()).expect_err("err");
        assert!(
            err.message.contains("Std.Task wait"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn array_map_is_vm_only() {
        let err = run_intrinsic(Intrinsic::Array(ArrayIntrinsic::Map), &mut Vec::new(), loc()).expect_err("err");
        assert!(
            err.message.contains("higher-order Std intrinsics"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn str_length_still_dispatches() {
        let mut stack = vec![Value::Str("ab".into())];
        run_intrinsic(Intrinsic::Str(StrIntrinsic::Length), &mut stack, loc()).unwrap();
        assert_eq!(stack, vec![Value::Integer(2)]);
    }
}