//! Dictionary entries in the fixed built-in operation catalog.
use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::Ty;

pub(super) const OPERATIONS: &[NativeOperation] = &[
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Length",
        implementation: "Std.Dictionaries.Length",
        signature: || function(vec![], Ty::Integer, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Number of entries",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "IsEmpty",
        implementation: "Std.Dictionaries.IsEmpty",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::IsEmpty,
        documentation: "Whether the entry count is zero",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "ContainsKey",
        implementation: "Std.Dictionaries.ContainsKey",
        signature: || {
            function(
                vec![p("Key", Ty::GenericParam("K".into(), None))],
                Ty::Boolean,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Key membership",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Get",
        implementation: "Std.Dictionaries.Get",
        signature: || {
            function(
                vec![p("Key", Ty::GenericParam("K".into(), None))],
                Ty::Option(Box::new(Ty::GenericParam("V".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Safe lookup, or `None`",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Keys",
        implementation: "Std.Dictionaries.Keys",
        signature: || {
            function(
                vec![],
                Ty::Array(Box::new(Ty::GenericParam("K".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Keys in insertion order",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Values",
        implementation: "Std.Dictionaries.Values",
        signature: || {
            function(
                vec![],
                Ty::Array(Box::new(Ty::GenericParam("V".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Values in insertion order",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Map",
        implementation: "Std.Dictionaries.Map",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("V", Ty::GenericParam("V".into(), None))],
                        Ty::GenericParam("V2".into(), None),
                        false,
                    )),
                )],
                Ty::Dict(
                    Box::new(Ty::GenericParam("K".into(), None)),
                    Box::new(Ty::GenericParam("V2".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Transform values while retaining keys",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Filter",
        implementation: "Std.Dictionaries.Filter",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![
                            p("K", Ty::GenericParam("K".into(), None)),
                            p("V", Ty::GenericParam("V".into(), None)),
                        ],
                        Ty::Boolean,
                        false,
                    )),
                )],
                Ty::Dict(
                    Box::new(Ty::GenericParam("K".into(), None)),
                    Box::new(Ty::GenericParam("V".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Keep matching entries",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Reduce",
        implementation: "Std.Dictionaries.Reduce",
        signature: || {
            function(
                vec![
                    p("Init", Ty::GenericParam("U".into(), None)),
                    p(
                        "F",
                        Ty::Function(function(
                            vec![
                                p("Acc", Ty::GenericParam("U".into(), None)),
                                p("Key", Ty::GenericParam("K".into(), None)),
                                p("Value", Ty::GenericParam("V".into(), None)),
                            ],
                            Ty::GenericParam("U".into(), None),
                            false,
                        )),
                    ),
                ],
                Ty::GenericParam("U".into(), None),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Fold entries in insertion order",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Merge",
        implementation: "Std.Dictionaries.Merge",
        signature: || {
            function(
                vec![p(
                    "D2",
                    Ty::Dict(
                        Box::new(Ty::GenericParam("K".into(), None)),
                        Box::new(Ty::GenericParam("V".into(), None)),
                    ),
                )],
                Ty::Dict(
                    Box::new(Ty::GenericParam("K".into(), None)),
                    Box::new(Ty::GenericParam("V".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Combine values; the argument wins on key conflicts",
    },
    NativeOperation {
        receiver: NativeReceiver::Dictionary,
        name: "Remove",
        implementation: "Std.Dictionaries.Remove",
        signature: || {
            function(
                vec![p("Key", Ty::GenericParam("K".into(), None))],
                Ty::Dict(
                    Box::new(Ty::GenericParam("K".into(), None)),
                    Box::new(Ty::GenericParam("V".into(), None)),
                ),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Return a value without the key",
    },
];
