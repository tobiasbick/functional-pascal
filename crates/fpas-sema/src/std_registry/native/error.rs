//! Automatic native-error output. Documentation: `docs/pascal/language/types/errors.md`.
use super::{NativeLowering, NativeOperation, NativeReceiver, function};
use crate::types::Ty;

pub(super) const OPERATIONS: &[NativeOperation] = &[NativeOperation {
    receiver: NativeReceiver::Error,
    name: "Message",
    implementation: "Std.Os.Errors.Message",
    signature: || function(vec![], Ty::String, false),
    lowering: NativeLowering::Intrinsic,
    documentation: "Return the automatic error text, including typed causes; performs no I/O",
}];
