//! Pending-intrinsic classification for recording capture.
//!
//! Unsupported host effects are named here so a capturing session can reject
//! them before dispatch. Recording-off execution does not use this filter.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use crate::{
    ConsoleIntrinsic, InstructionAddress, Intrinsic, Opcode, TimeIntrinsic, VerifiedExecutable,
};
use fpas_diagnostics::{Diagnostic, codes::RUNTIME_RECORDING_UNSUPPORTED_EFFECT};

/// Return a diagnostic when the pending instruction is an unsupported host effect.
///
/// The instruction is not executed. `None` leaves dispatch unchanged.
pub(crate) fn pending_unsupported_recording_effect(
    executable: &VerifiedExecutable,
    ip: usize,
) -> Option<(InstructionAddress, Diagnostic)> {
    let image = executable.executable();
    let instruction = image.code.get(ip).copied()?;
    if instruction.opcode().ok()? != Opcode::Intrinsic {
        return None;
    }
    let intrinsic = Intrinsic::from_u16(instruction.abc_operands().ok()?.b)?;
    let unit = unsupported_recording_unit(intrinsic)?;
    let address = InstructionAddress::try_from_index(ip).ok()?;
    Some((
        address,
        *crate::execution::vm::diagnostics::at_address(
            image,
            address,
            RUNTIME_RECORDING_UNSUPPORTED_EFFECT,
            format!("recording capture cannot execute {unit}"),
            "Capture records all-stop events and queued ReadText/ReadLn only. Continue without record, or avoid this host effect while capturing. Reverse execution stays unavailable.",
        ),
    ))
}

const fn unsupported_recording_unit(intrinsic: Intrinsic) -> Option<&'static str> {
    match intrinsic {
        Intrinsic::Bytes(_) => None,
        Intrinsic::Mcp(_) => {
            if crate::intrinsic_debug_effects(intrinsic).is_debug_safe() {
                None
            } else {
                Some("Std.Ai.Mcp")
            }
        }
        Intrinsic::Server(_) => Some("Std.Net.Server"),
        Intrinsic::Random(_) => Some("Std.Math.Random"),
        Intrinsic::Crypto(_) => Some("Std.Crypto"),
        Intrinsic::Time(TimeIntrinsic::Sleep) => None,
        Intrinsic::Time(_) => Some("Std.Time"),
        Intrinsic::Fs(_) => Some("Std.Io.Fs"),
        Intrinsic::Env(_) => Some("Std.Os.Env"),
        Intrinsic::Proc(
            crate::ProcIntrinsic::DefaultOptions | crate::ProcIntrinsic::DefaultStartOptions,
        ) => None,
        Intrinsic::Proc(_) => Some("Std.Os.Proc"),
        Intrinsic::Net(_) => Some("Std.Net.Sockets"),
        Intrinsic::Http(_) => Some("Std.Net.Http"),
        Intrinsic::Args(_) => Some("Std.Os.Args"),
        Intrinsic::Console(
            ConsoleIntrinsic::Read
            | ConsoleIntrinsic::ReadLn
            | ConsoleIntrinsic::Write
            | ConsoleIntrinsic::WriteLn,
        ) => None,
        Intrinsic::Console(_) => Some("Std.Console"),
        Intrinsic::Acp(_)
        | Intrinsic::Str(_)
        | Intrinsic::Conv(_)
        | Intrinsic::Encoding(_)
        | Intrinsic::Tokenizer(_)
        | Intrinsic::Parse(_)
        | Intrinsic::Math(_)
        | Intrinsic::Bits(_)
        | Intrinsic::Path(_)
        | Intrinsic::Json(_)
        | Intrinsic::Toml(_)
        | Intrinsic::Array(_)
        | Intrinsic::Dict(_)
        | Intrinsic::Result(_)
        | Intrinsic::Option(_)
        | Intrinsic::Task(_)
        | Intrinsic::Test(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ArgsIntrinsic, FsIntrinsic, MathIntrinsic, PathIntrinsic, RandomIntrinsic, TimeIntrinsic,
    };

    #[test]
    fn capture_rejects_host_nondeterminism_and_keeps_safe_intrinsics() {
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Random(RandomIntrinsic::RandomInt)),
            Some("Std.Math.Random")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Time(TimeIntrinsic::TimestampMillis)),
            Some("Std.Time")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Fs(FsIntrinsic::ReadText)),
            Some("Std.Io.Fs")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Args(ArgsIntrinsic::ParamCount)),
            Some("Std.Os.Args")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Console(ConsoleIntrinsic::ReadKey)),
            Some("Std.Console")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Time(TimeIntrinsic::Sleep)),
            None
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Console(ConsoleIntrinsic::WriteLn)),
            None
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Math(MathIntrinsic::Sqrt)),
            None
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Path(PathIntrinsic::Join)),
            None
        );
    }

    #[test]
    fn every_intrinsic_is_classified_for_recording_capture() {
        for intrinsic in Intrinsic::all() {
            let _ = unsupported_recording_unit(intrinsic);
        }
    }
}
