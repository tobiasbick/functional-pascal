//! Canonical byte-buffer receiver operations and typed mutation results.

use super::{NativeLowering, NativeOperation, NativeReceiver, function, p};
use crate::types::{EnumTy, EnumVariantTy, Ty};
use std::sync::{Arc, OnceLock};

fn error_type() -> Ty {
    static ERROR: OnceLock<Ty> = OnceLock::new();
    ERROR
        .get_or_init(|| {
            let family = fpas_builtins::native_errors::NativeErrorFamily::Buffer;
            Ty::Enum(Arc::new(EnumTy {
                is_error: true,
                name: family.name().into(),
                type_params: vec![],
                type_args: vec![],
                variants: family
                    .variants()
                    .iter()
                    .map(|variant| EnumVariantTy {
                        name: variant.name.into(),
                        message: Some(variant.message.into()),
                        backing_value: None,
                        fields: variant
                            .fields
                            .iter()
                            .map(|(name, _)| (name.to_string(), Ty::Integer))
                            .collect(),
                    })
                    .collect(),
            }))
        })
        .clone()
}

fn result(ty: Ty) -> Ty {
    Ty::Result(Box::new(ty), Box::new(error_type()))
}

macro_rules! operation {
    ($name:literal, $result:expr, [$($label:literal => $ty:expr),*], $lowering:expr, $doc:literal) => {
        NativeOperation {
            receiver: NativeReceiver::ByteBuffer,
            name: $name,
            implementation: concat!("Std.Bytes.", $name),
            signature: || function(vec![$(p($label, $ty)),*], $result, false),
            lowering: $lowering,
            documentation: $doc,
        }
    };
}

macro_rules! mutate {
    ($name:literal, $base:literal, [$($label:literal => $ty:expr),*]) => {
        operation!($name, result(Ty::Boolean), [$($label => $ty),*], NativeLowering::InPlace(concat!("Std.Bytes.", $base)), "Update writable byte storage on success; preserve its value and capacity on failure")
    };
}

pub(super) const OPERATIONS: &[NativeOperation] = &[
    operation!(
        "Length",
        Ty::Integer,
        [],
        NativeLowering::Intrinsic,
        "Logical byte count"
    ),
    operation!(
        "IsEmpty",
        Ty::Boolean,
        [],
        NativeLowering::IsEmpty,
        "Whether the byte count is zero"
    ),
    operation!(
        "Capacity",
        Ty::Integer,
        [],
        NativeLowering::Intrinsic,
        "Reserved total byte capacity"
    ),
    operation!(
        "ToArray",
        result(Ty::Array(Box::new(Ty::Integer))),
        [],
        NativeLowering::Intrinsic,
        "Copy bytes into ordinary integer elements"
    ),
    operation!("Slice", result(Ty::ByteBuffer), ["Start" => Ty::Integer, "Len" => Ty::Integer], NativeLowering::Intrinsic, "Copy a checked byte range; a full range may share immutable storage"),
    operation!("Concat", result(Ty::ByteBuffer), ["B" => Ty::ByteBuffer], NativeLowering::Intrinsic, "Append another buffer to a new value"),
    operation!("SetAt", result(Ty::ByteBuffer), ["Index" => Ty::Integer, "Value" => Ty::Integer], NativeLowering::Intrinsic, "Replace one checked byte in a new value"),
    operation!("Fill", result(Ty::ByteBuffer), ["Value" => Ty::Integer, "Start" => Ty::Integer, "Len" => Ty::Integer], NativeLowering::Intrinsic, "Fill an existing byte range in a new value"),
    operation!("CopyFrom", result(Ty::ByteBuffer), ["Source" => Ty::ByteBuffer, "SourceStart" => Ty::Integer, "TargetStart" => Ty::Integer, "Len" => Ty::Integer], NativeLowering::Intrinsic, "Copy a source range over an existing destination range"),
    operation!("Resize", result(Ty::ByteBuffer), ["Length" => Ty::Integer], NativeLowering::Intrinsic, "Resize a new value, retaining capacity and zeroing exposed bytes"),
    mutate!("SliceInPlace", "Slice", ["Start" => Ty::Integer, "Len" => Ty::Integer]),
    mutate!("ConcatInPlace", "Concat", ["B" => Ty::ByteBuffer]),
    mutate!("SetAtInPlace", "SetAt", ["Index" => Ty::Integer, "Value" => Ty::Integer]),
    mutate!("FillInPlace", "Fill", ["Value" => Ty::Integer, "Start" => Ty::Integer, "Len" => Ty::Integer]),
    mutate!("CopyFromInPlace", "CopyFrom", ["Source" => Ty::ByteBuffer, "SourceStart" => Ty::Integer, "TargetStart" => Ty::Integer, "Len" => Ty::Integer]),
    mutate!("ResizeInPlace", "Resize", ["Length" => Ty::Integer]),
    mutate!("ReserveInPlace", "Reserve", ["Capacity" => Ty::Integer]),
    mutate!("TrimCapacityInPlace", "TrimCapacity", []),
];
