//! Result entries in the fixed built-in operation catalog.
use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::Ty;

pub(super) const OPERATIONS: &[NativeOperation] = &[
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "IsOk",
        implementation: "Std.Results.IsOk",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether the result is successful",
    },
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "IsError",
        implementation: "Std.Results.IsError",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether the result is an error",
    },
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "Map",
        implementation: "Std.Results.Map",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("V", Ty::GenericParam("T".into(), None))],
                        Ty::GenericParam("U".into(), None),
                        false,
                    )),
                )],
                Ty::Result(
                    Box::new(Ty::GenericParam("U".into(), None)),
                    Box::new(Ty::GenericParam("E".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Transform the successful value",
    },
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "AndThen",
        implementation: "Std.Results.AndThen",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("V", Ty::GenericParam("T".into(), None))],
                        Ty::Result(
                            Box::new(Ty::GenericParam("U".into(), None)),
                            Box::new(Ty::GenericParam("E".into(), None)),
                        ),
                        false,
                    )),
                )],
                Ty::Result(
                    Box::new(Ty::GenericParam("U".into(), None)),
                    Box::new(Ty::GenericParam("E".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Chain an operation with the same error type",
    },
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "OrElse",
        implementation: "Std.Results.OrElse",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("Err", Ty::GenericParam("E".into(), None))],
                        Ty::Result(
                            Box::new(Ty::GenericParam("T".into(), None)),
                            Box::new(Ty::GenericParam("E2".into(), None)),
                        ),
                        false,
                    )),
                )],
                Ty::Result(
                    Box::new(Ty::GenericParam("T".into(), None)),
                    Box::new(Ty::GenericParam("E2".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Invoke error recovery; may change the error type",
    },
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "Unwrap",
        implementation: "Std.Results.Unwrap",
        signature: || function(vec![], Ty::GenericParam("T".into(), None), false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Extract the value; panic for `Error`",
    },
    NativeOperation {
        receiver: NativeReceiver::Result,
        name: "UnwrapOr",
        implementation: "Std.Results.UnwrapOr",
        signature: || {
            function(
                vec![p("Default", Ty::GenericParam("T".into(), None))],
                Ty::GenericParam("T".into(), None),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Extract the value or use the default",
    },
];
