//! Explicit mutating counterparts of type-preserving collection operations.
//! Documentation: `docs/pascal/language/types/collection-mutation.md`.

use super::{NativeLowering, NativeOperation, NativeReceiver};
use crate::types::{FunctionTy, Ty};
use std::collections::HashMap;

fn signature(entries: &[NativeOperation], name: &str) -> FunctionTy {
    let entry = entries
        .iter()
        .find(|entry| entry.name == name)
        .expect("value operation");
    let bindings = HashMap::from([
        ("u".into(), Ty::GenericParam("T".into(), None)),
        ("v2".into(), Ty::GenericParam("V".into(), None)),
    ]);
    let Ty::Function(mut signature) =
        crate::types::substitute_type_parameters(&Ty::Function((entry.signature)()), &bindings)
    else {
        unreachable!()
    };
    signature.return_type = Box::new(if name == "Pop" {
        Ty::GenericParam("T".into(), None)
    } else {
        Ty::Unit
    });
    signature
}

macro_rules! operations {
    ($($module:ident, $receiver:ident, $unit:literal, $name:literal, $implementation:literal;)*) => {
        pub(super) const OPERATIONS: &[NativeOperation] = &[$(
            NativeOperation {
                receiver: NativeReceiver::$receiver,
                name: concat!($name, "InPlace"),
                implementation: concat!($unit, ".", $implementation, "InPlace"),
                signature: || signature(super::$module::OPERATIONS, $name),
                lowering: NativeLowering::InPlace(concat!($unit, ".", $implementation)),
                documentation: concat!("Update writable receiver with ", $name, "; preserve its type and other value copies"),
            },
        )*];
    };
}

operations! {
    array, Array, "Std.Arrays", "Sort", "Sort";
    array, Array, "Std.Arrays", "Reverse", "Reverse";
    array, Array, "Std.Arrays", "Filter", "Filter";
    array, Array, "Std.Arrays", "Slice", "Slice";
    array, Array, "Std.Arrays", "Concat", "Concat";
    array, Array, "Std.Arrays", "Push", "Push";
    array, Array, "Std.Arrays", "Pop", "Pop";
    array, Array, "Std.Arrays", "Map", "Map";
    array, Array, "Std.Arrays", "FlatMap", "FlatMap";
    dictionary, Dictionary, "Std.Dictionaries", "Remove", "Remove";
    dictionary, Dictionary, "Std.Dictionaries", "Merge", "Merge";
    dictionary, Dictionary, "Std.Dictionaries", "Filter", "Filter";
    dictionary, Dictionary, "Std.Dictionaries", "Map", "Map";
    string, String, "Std.Str", "Trim", "Trim";
    string, String, "Std.Str", "TrimLeft", "TrimLeft";
    string, String, "Std.Str", "TrimRight", "TrimRight";
    string, String, "Std.Str", "ToUpper", "ToUpper";
    string, String, "Std.Str", "ToLower", "ToLower";
    string, String, "Std.Str", "Replace", "Replace";
    string, String, "Std.Str", "Map", "Map";
    string, String, "Std.Str", "Filter", "Filter";
    string, String, "Std.Str", "Slice", "Substring";
    string, String, "Std.Str", "RepeatStr", "RepeatStr";
    string, String, "Std.Str", "PadLeft", "PadLeft";
    string, String, "Std.Str", "PadRight", "PadRight";
    string, String, "Std.Str", "PadCenter", "PadCenter";
    string, String, "Std.Str", "SetCharAt", "SetCharAt";
    string, String, "Std.Str", "Insert", "Insert";
    string, String, "Std.Str", "Delete", "Delete";
    string, String, "Std.Str", "Reverse", "Reverse";
    string, String, "Std.Str", "Format", "Format";
}
