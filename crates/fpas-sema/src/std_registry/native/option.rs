//! Option entries in the fixed built-in operation catalog.
use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::Ty;

pub(super) const OPERATIONS: &[NativeOperation] = &[
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "IsSome",
        implementation: "Std.Options.IsSome",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether a value is present",
    },
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "IsNone",
        implementation: "Std.Options.IsNone",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether the option is absent",
    },
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "Map",
        implementation: "Std.Options.Map",
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
                Ty::Option(Box::new(Ty::GenericParam("U".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Transform the present value",
    },
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "AndThen",
        implementation: "Std.Options.AndThen",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("V", Ty::GenericParam("T".into(), None))],
                        Ty::Option(Box::new(Ty::GenericParam("U".into(), None))),
                        false,
                    )),
                )],
                Ty::Option(Box::new(Ty::GenericParam("U".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Chain an optional operation",
    },
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "OrElse",
        implementation: "Std.Options.OrElse",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![],
                        Ty::Option(Box::new(Ty::GenericParam("T".into(), None))),
                        false,
                    )),
                )],
                Ty::Option(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Invoke a fallback only for `None`",
    },
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "Unwrap",
        implementation: "Std.Options.Unwrap",
        signature: || function(vec![], Ty::GenericParam("T".into(), None), false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Extract the value; panic for `None`",
    },
    NativeOperation {
        receiver: NativeReceiver::Option,
        name: "UnwrapOr",
        implementation: "Std.Options.UnwrapOr",
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
