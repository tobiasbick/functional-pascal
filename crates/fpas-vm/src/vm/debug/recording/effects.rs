//! Pending-intrinsic classification for recording capture.
//!
//! Unsupported host effects are named here so a capturing session can reject
//! them before dispatch. Recording-off execution does not use this filter.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{
    ConsoleIntrinsic, InstructionAddress, Intrinsic, Opcode, TimeIntrinsic, VerifiedExecutable,
};
use fpas_diagnostics::{Diagnostic, SourceSpan, codes::RUNTIME_RECORDING_UNSUPPORTED_EFFECT};

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
    let span = image.source_map.lookup(address).map_or_else(
        || SourceSpan::new(0, 1, 1, 1),
        |run| SourceSpan::new_with_source(0, 1, run.line, run.column, run.source.get()),
    );
    Some((
        address,
        Diagnostic::error(
            RUNTIME_RECORDING_UNSUPPORTED_EFFECT,
            format!("recording capture cannot execute {unit}"),
            Some(
                "Capture records all-stop events and queued Read/ReadLn only. Continue without record, or avoid this host effect while capturing. Reverse execution stays unavailable."
                    .to_string(),
            ),
            span,
        ),
    ))
}

const fn unsupported_recording_unit(intrinsic: Intrinsic) -> Option<&'static str> {
    match intrinsic {
        Intrinsic::Random(_) => Some("Std.Random"),
        Intrinsic::Time(TimeIntrinsic::Sleep) => None,
        Intrinsic::Time(_) => Some("Std.Time"),
        Intrinsic::Fs(_) => Some("Std.Fs"),
        Intrinsic::Env(_) => Some("Std.Env"),
        Intrinsic::Proc(_) => Some("Std.Proc"),
        Intrinsic::Graph(_) => Some("Std.Graph"),
        Intrinsic::Args(_) => Some("Std.Args"),
        Intrinsic::Console(
            ConsoleIntrinsic::Read
            | ConsoleIntrinsic::ReadLn
            | ConsoleIntrinsic::Write
            | ConsoleIntrinsic::WriteLn,
        ) => None,
        Intrinsic::Console(_) => Some("Std.Console"),
        Intrinsic::Str(_)
        | Intrinsic::Conv(_)
        | Intrinsic::Parse(_)
        | Intrinsic::Math(_)
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
    use fpas_bytecode::{
        ArgsIntrinsic, FsIntrinsic, MathIntrinsic, PathIntrinsic, RandomIntrinsic, TimeIntrinsic,
    };

    #[test]
    fn capture_rejects_host_nondeterminism_and_keeps_safe_intrinsics() {
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Random(RandomIntrinsic::RandomInt)),
            Some("Std.Random")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Time(TimeIntrinsic::TimestampMillis)),
            Some("Std.Time")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Fs(FsIntrinsic::ReadText)),
            Some("Std.Fs")
        );
        assert_eq!(
            unsupported_recording_unit(Intrinsic::Args(ArgsIntrinsic::ParamCount)),
            Some("Std.Args")
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
