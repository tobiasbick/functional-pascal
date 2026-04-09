//! Dispatches `Op::Intrinsic` to unit modules (`str`, `conv`, `math`, `array`, `result_option`, `dict`).
//! Console, TUI, and `Std.Task` (`TaskWait`, `TaskWaitAll`) intrinsics are handled in `fpas-vm`, not here.
//!
//! **Documentation:** `docs/pascal/std/README.md` (from the repository root).
//! **Maintenance:** When adding or rerouting intrinsics, update the README, the relevant unit `.md` file, and `fpas-bytecode::Intrinsic`.

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
            "internal: Std.Console and Std.Tui session intrinsics are handled in the VM",
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
        "This indicates a VM dispatch bug: console/TUI intrinsics must be handled in the VM, and all other std opcodes must route through fpas-std. Please report this as a compiler/runtime issue.",
        location,
    ))
}
