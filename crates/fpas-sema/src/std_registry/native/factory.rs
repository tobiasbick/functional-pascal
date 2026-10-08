//! Factory entries in the fixed built-in operation catalog.
use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::Ty;

pub(super) const OPERATIONS: &[NativeOperation] = &[
    NativeOperation {
        receiver: NativeReceiver::StringFactory,
        name: "Chr",
        implementation: "Std.Str.Chr",
        signature: || function(vec![p("N", Ty::Integer)], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Construct one scalar from a valid Unicode codepoint",
    },
    NativeOperation {
        receiver: NativeReceiver::ArrayFactory,
        name: "Fill",
        implementation: "Std.Arrays.Fill",
        signature: || {
            function(
                vec![
                    p("Value", Ty::GenericParam("T".into(), None)),
                    p("Count", Ty::Integer),
                ],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Construct an array of repeated values",
    },
];
