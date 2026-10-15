//! Array entries in the fixed built-in operation catalog.
use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::{ProcedureTy, Ty};

pub(super) const OPERATIONS: &[NativeOperation] = &[
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Length",
        implementation: "Std.Arrays.Length",
        signature: || function(vec![], Ty::Integer, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Number of elements",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "IsEmpty",
        implementation: "Std.Arrays.IsEmpty",
        signature: || function(vec![], Ty::Boolean, false),
        lowering: NativeLowering::IsEmpty,
        documentation: "Whether the element count is zero",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "IndexOf",
        implementation: "Std.Arrays.IndexOf",
        signature: || {
            function(
                vec![p("Value", Ty::GenericParam("T".into(), None))],
                Ty::Option(Box::new(Ty::Integer)),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Some(first matching index), or None",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Map",
        implementation: "Std.Arrays.Map",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::GenericParam("U".into(), None),
                        false,
                    )),
                )],
                Ty::Array(Box::new(Ty::GenericParam("U".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Transform each element",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Filter",
        implementation: "Std.Arrays.Filter",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::Boolean,
                        false,
                    )),
                )],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Keep matching elements",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Reduce",
        implementation: "Std.Arrays.Reduce",
        signature: || {
            function(
                vec![
                    p("Init", Ty::GenericParam("U".into(), None)),
                    p(
                        "F",
                        Ty::Function(function(
                            vec![
                                p("Acc", Ty::GenericParam("U".into(), None)),
                                p("V", Ty::GenericParam("T".into(), None)),
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
        documentation: "Fold elements left to right",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Find",
        implementation: "Std.Arrays.Find",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::Boolean,
                        false,
                    )),
                )],
                Ty::Option(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "First matching element, or `None`",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "FindIndex",
        implementation: "Std.Arrays.FindIndex",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::Boolean,
                        false,
                    )),
                )],
                Ty::Option(Box::new(Ty::Integer)),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Some(first matching index), or None",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Any",
        implementation: "Std.Arrays.Any",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::Boolean,
                        false,
                    )),
                )],
                Ty::Boolean,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether any element satisfies the predicate",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "All",
        implementation: "Std.Arrays.All",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::Boolean,
                        false,
                    )),
                )],
                Ty::Boolean,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Whether every element satisfies the predicate",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Sort",
        implementation: "Std.Arrays.Sort",
        signature: || {
            function(
                vec![],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Return a stable ascending value; elements require Comparable (including payload-free enums)",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "SortBy",
        implementation: "Std.Arrays.SortBy",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::GenericParam("K".into(), None),
                        false,
                    )),
                )],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Stable ascending sort by Comparable keys; evaluate F once per element in input order",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Reverse",
        implementation: "Std.Arrays.Reverse",
        signature: || {
            function(
                vec![],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Return a reversed value",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Slice",
        implementation: "Std.Arrays.Slice",
        signature: || {
            function(
                vec![p("Start", Ty::Integer), p("Len", Ty::Integer)],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Return the checked subrange",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Concat",
        implementation: "Std.Arrays.Concat",
        signature: || {
            function(
                vec![p(
                    "B",
                    Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                )],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Append the second array's elements to the result",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "FlatMap",
        implementation: "Std.Arrays.FlatMap",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Function(function(
                        vec![p("X", Ty::GenericParam("T".into(), None))],
                        Ty::Array(Box::new(Ty::GenericParam("U".into(), None))),
                        false,
                    )),
                )],
                Ty::Array(Box::new(Ty::GenericParam("U".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Map each element, then flatten the results",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "ForEach",
        implementation: "Std.Arrays.ForEach",
        signature: || {
            function(
                vec![p(
                    "F",
                    Ty::Procedure(ProcedureTy {
                        type_params: vec![],
                        params: vec![p("X", Ty::GenericParam("T".into(), None))],
                        variadic: false,
                    }),
                )],
                Ty::Unit,
                false,
            )
        },
        lowering: NativeLowering::Intrinsic,
        documentation: "Invoke a procedure per element; only a final chain step",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Push",
        implementation: "Std.Arrays.Push",
        signature: || {
            function(
                vec![p("Value", Ty::GenericParam("T".into(), None))],
                Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                false,
            )
        },
        lowering: NativeLowering::ArrayPush,
        documentation: "Return a new appended array; preserve the receiver",
    },
    NativeOperation {
        receiver: NativeReceiver::Array,
        name: "Pop",
        implementation: "Std.Arrays.Pop",
        signature: || {
            function(
                vec![],
                crate::types::array_pop_result(Ty::GenericParam("T".into(), None)),
                false,
            )
        },
        lowering: NativeLowering::ArrayPop,
        documentation: "Return Remaining and Value; preserve the receiver; empty arrays raise a runtime error",
    },
    NativeOperation {
        receiver: NativeReceiver::StringArray,
        name: "Join",
        implementation: "Std.Str.Join",
        signature: || function(vec![p("Delim", Ty::String)], Ty::String, false),
        lowering: NativeLowering::Intrinsic,
        documentation: "Join the receiver's strings with the delimiter",
    },
];
