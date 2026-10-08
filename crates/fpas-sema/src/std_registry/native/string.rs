//! String entries in the fixed built-in operation catalog.
use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::Ty;

pub(super) const OPERATIONS: &[NativeOperation] = &[
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Length",
        implementation: "Std.Str.Length",
        signature: || function(vec![], Ty::Integer, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Number of Unicode scalars",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "IsEmpty",
        implementation: "Std.Str.IsEmpty",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::IsEmpty,
        documentation: "Whether the scalar count is zero",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Contains",
        implementation: "Std.Str.Contains",
        signature: || function(vec![p("Sub", Ty::String)], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Substring membership",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "StartsWith",
        implementation: "Std.Str.StartsWith",
        signature: || function(vec![p("Pre", Ty::String)], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Prefix test",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "EndsWith",
        implementation: "Std.Str.EndsWith",
        signature: || function(vec![p("Suf", Ty::String)], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Suffix test",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "IndexOf",
        implementation: "Std.Str.IndexOf",
        signature: || function(vec![p("Sub", Ty::String)], Ty::Integer, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "First scalar index, or `-1`",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Trim",
        implementation: "Std.Str.Trim",
        signature: || function(vec![], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Remove leading and trailing whitespace",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "ToUpper",
        implementation: "Std.Str.ToUpper",
        signature: || function(vec![], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Uppercased value",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "ToLower",
        implementation: "Std.Str.ToLower",
        signature: || function(vec![], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Lowercased value",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Replace",
        implementation: "Std.Str.Replace",
        signature: || {
            function(
                vec![p("Old", Ty::String), p("New", Ty::String)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Replace all occurrences",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Split",
        implementation: "Std.Str.Split",
        signature: || {
            function(
                vec![p("Delim", Ty::String)],
                Ty::Array(Box::new(Ty::String)),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Split into segments",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Map",
        implementation: "Std.Str.Map",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(vec![p("C", Ty::String)], Ty::String, false)),
                )],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Transform each scalar into exactly one scalar",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Filter",
        implementation: "Std.Str.Filter",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(vec![p("C", Ty::String)], Ty::Boolean, false)),
                )],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Keep matching scalars",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Reduce",
        implementation: "Std.Str.Reduce",
        signature: || {
            function(
                vec![
                    p("Init", Ty::GenericParam("U".into(), None)),
                    p(
                        "F",
                        Ty::Function(function(
                            vec![
                                p("Acc", Ty::GenericParam("U".into(), None)),
                                p("C", Ty::String),
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
        documentation: "Fold scalars left to right",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Slice",
        implementation: "Std.Str.Substring",
        signature: || {
            function(
                vec![p("Start", Ty::Integer), p("Len", Ty::Integer)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Return the checked scalar range; same public name and argument roles as array `Slice`",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "LastIndexOf",
        implementation: "Std.Str.LastIndexOf",
        signature: || function(vec![p("Sub", Ty::String)], Ty::Integer, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Last matching scalar index, or `-1`",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "IsNumeric",
        implementation: "Std.Str.IsNumeric",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether the value matches the existing numeric-text rules",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "RepeatStr",
        implementation: "Std.Str.RepeatStr",
        signature: || function(vec![p("N", Ty::Integer)], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Repeat the complete string",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "PadLeft",
        implementation: "Std.Str.PadLeft",
        signature: || {
            function(
                vec![p("Width", Ty::Integer), p("PadChar", Ty::String)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Pad on the left",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "PadRight",
        implementation: "Std.Str.PadRight",
        signature: || {
            function(
                vec![p("Width", Ty::Integer), p("PadChar", Ty::String)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Pad on the right",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "PadCenter",
        implementation: "Std.Str.PadCenter",
        signature: || {
            function(
                vec![p("Width", Ty::Integer), p("PadChar", Ty::String)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Center within the padded width",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "FromChar",
        implementation: "Std.Str.FromChar",
        signature: || function(vec![p("N", Ty::Integer)], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Repeat a receiver that must contain exactly one scalar",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "CharAt",
        implementation: "Std.Str.CharAt",
        signature: || function(vec![p("Index", Ty::Integer)], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Read the scalar at the checked index",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "SetCharAt",
        implementation: "Std.Str.SetCharAt",
        signature: || {
            function(
                vec![p("Index", Ty::Integer), p("C", Ty::String)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Return a value with one scalar replaced",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Ord",
        implementation: "Std.Str.Ord",
        signature: || function(vec![], Ty::Integer, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Unicode codepoint of a receiver that must contain exactly one scalar",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Insert",
        implementation: "Std.Str.Insert",
        signature: || {
            function(
                vec![p("Index", Ty::Integer), p("Sub", Ty::String)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Insert text at the scalar index",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Delete",
        implementation: "Std.Str.Delete",
        signature: || {
            function(
                vec![p("Index", Ty::Integer), p("Len", Ty::Integer)],
                Ty::String,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Remove the checked scalar range",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Reverse",
        implementation: "Std.Str.Reverse",
        signature: || function(vec![], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Return reversed scalars",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "TrimLeft",
        implementation: "Std.Str.TrimLeft",
        signature: || function(vec![], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Remove leading whitespace",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "TrimRight",
        implementation: "Std.Str.TrimRight",
        signature: || function(vec![], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Remove trailing whitespace",
    },
    NativeOperation {
        receiver: NativeReceiver::String,
        name: "Format",
        implementation: "Std.Str.Format",
        signature: || function(vec![], Ty::String, true),
        lowering: NativeLowering::Intrinsic,
        documentation: "Use the receiver as the format template; heterogeneous variadic arguments are positional only",
    },
];
