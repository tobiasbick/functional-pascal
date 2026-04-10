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
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
/// Execute a standard-library intrinsic; mutates `stack` (Pascal call order: args already pushed).
pub fn run_intrinsic(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<(), StdError> {
    if matches!(
        intrinsic,
        Intrinsic::ConsoleReadLn
            | Intrinsic::ConsoleRead
            | Intrinsic::ConsoleReadKey
            | Intrinsic::ConsoleKeyPressed
            | Intrinsic::ConsoleReadKeyEvent
            | Intrinsic::ConsoleClrScr
            | Intrinsic::ConsoleClrEol
            | Intrinsic::ConsoleGotoXY
            | Intrinsic::ConsoleWhereX
            | Intrinsic::ConsoleWhereY
            | Intrinsic::ConsoleWindMin
            | Intrinsic::ConsoleWindMax
            | Intrinsic::ConsoleDelLine
            | Intrinsic::ConsoleInsLine
            | Intrinsic::ConsoleWindow
            | Intrinsic::ConsoleTextColor
            | Intrinsic::ConsoleTextBackground
            | Intrinsic::ConsoleHighVideo
            | Intrinsic::ConsoleLowVideo
            | Intrinsic::ConsoleNormVideo
            | Intrinsic::ConsoleTextAttr
            | Intrinsic::ConsoleSetTextAttr
            | Intrinsic::ConsoleDelay
            | Intrinsic::ConsoleCursorOn
            | Intrinsic::ConsoleCursorBig
            | Intrinsic::ConsoleCursorOff
            | Intrinsic::ConsoleTextMode
            | Intrinsic::ConsoleLastMode
            | Intrinsic::ConsoleScreenWidth
            | Intrinsic::ConsoleScreenHeight
            | Intrinsic::ConsoleSound
            | Intrinsic::ConsoleNoSound
            | Intrinsic::ConsoleAssignCrt
            | Intrinsic::ConsoleEventPending
            | Intrinsic::ConsoleReadEvent
            | Intrinsic::ConsoleEnableRawMode
            | Intrinsic::ConsoleDisableRawMode
            | Intrinsic::ConsoleEnterAltScreen
            | Intrinsic::ConsoleLeaveAltScreen
            | Intrinsic::ConsoleEnableMouse
            | Intrinsic::ConsoleDisableMouse
            | Intrinsic::ConsoleEnableFocus
            | Intrinsic::ConsoleDisableFocus
            | Intrinsic::ConsoleEnablePaste
            | Intrinsic::ConsoleDisablePaste
            | Intrinsic::ConsoleReadEventTimeout
            | Intrinsic::ConsolePollEvent
            | Intrinsic::ConsoleTextColorRGB
            | Intrinsic::ConsoleTextBackgroundRGB
            | Intrinsic::ConsoleTextColor256
            | Intrinsic::ConsoleTextBackground256
            | Intrinsic::TuiApplicationOpen
            | Intrinsic::TuiApplicationClose
            | Intrinsic::TuiApplicationSize
            | Intrinsic::TuiApplicationReadEvent
            | Intrinsic::TuiApplicationReadEventTimeout
            | Intrinsic::TuiApplicationPollEvent
            | Intrinsic::TuiApplicationRequestRedraw
            | Intrinsic::TuiApplicationRedrawPending
    ) {
        return Err(std_internal_error(
            "internal: Std.Console and Std.Tui intrinsics are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    if matches!(intrinsic, Intrinsic::TaskWait | Intrinsic::TaskWaitAll) {
        return Err(std_internal_error(
            "internal: Std.Task wait intrinsics (Wait, WaitAll) are handled in the VM",
            "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
            location,
        ));
    }

    if matches!(
        intrinsic,
        Intrinsic::ArrayMap
            | Intrinsic::ArrayFilter
            | Intrinsic::ArrayReduce
            | Intrinsic::ArrayFind
            | Intrinsic::ArrayFindIndex
            | Intrinsic::ArrayAny
            | Intrinsic::ArrayAll
            | Intrinsic::ArrayFlatMap
            | Intrinsic::ArrayForEach
            | Intrinsic::ResultMap
            | Intrinsic::ResultAndThen
            | Intrinsic::ResultOrElse
            | Intrinsic::OptionMap
            | Intrinsic::OptionAndThen
            | Intrinsic::OptionOrElse
            | Intrinsic::DictMap
            | Intrinsic::DictFilter
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
    use fpas_bytecode::{Intrinsic, SourceLocation, Value};

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn console_poll_event_is_vm_only() {
        let err = run_intrinsic(Intrinsic::ConsolePollEvent, &mut Vec::new(), loc())
            .expect_err("expected internal error");
        assert!(
            err.message.contains("Std.Console and Std.Tui"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn task_wait_is_vm_only() {
        let err = run_intrinsic(Intrinsic::TaskWait, &mut Vec::new(), loc()).expect_err("err");
        assert!(
            err.message.contains("Std.Task wait"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn array_map_is_vm_only() {
        let err = run_intrinsic(Intrinsic::ArrayMap, &mut Vec::new(), loc()).expect_err("err");
        assert!(
            err.message.contains("higher-order Std intrinsics"),
            "message={}",
            err.message
        );
    }

    #[test]
    fn str_length_still_dispatches() {
        let mut stack = vec![Value::Str("ab".into())];
        run_intrinsic(Intrinsic::StrLength, &mut stack, loc()).unwrap();
        assert_eq!(stack, vec![Value::Integer(2)]);
    }
}
