//! Completion of unknown payloads during generic argument inference.
//! See `docs/pascal/language/types/generics.md`.

use super::Ty;

/// Complete holes in mutually compatible evidence without promoting concrete types.
pub(crate) fn merge_inferred_types(left: &Ty, right: &Ty) -> Ty {
    let merge = |left: &Ty, right: &Ty| Box::new(merge_inferred_types(left, right));
    match (left, right) {
        (Ty::Error, _) => right.clone(),
        (Ty::Array(left), Ty::Array(right)) => Ty::Array(merge(left, right)),
        (Ty::Option(left), Ty::Option(right)) => Ty::Option(merge(left, right)),
        (Ty::Channel(left), Ty::Channel(right)) => Ty::Channel(merge(left, right)),
        (Ty::Task(left), Ty::Task(right)) => Ty::Task(merge(left, right)),
        (Ty::Result(left_ok, left_error), Ty::Result(right_ok, right_error)) => {
            Ty::Result(merge(left_ok, right_ok), merge(left_error, right_error))
        }
        (Ty::Dict(left_key, left_value), Ty::Dict(right_key, right_value)) => {
            Ty::Dict(merge(left_key, right_key), merge(left_value, right_value))
        }
        (Ty::Enum(left), Ty::Enum(right)) => {
            let mut enumeration = left.as_ref().clone();
            enumeration.type_args = left
                .type_args
                .iter()
                .zip(&right.type_args)
                .map(|(left, right)| merge_inferred_types(left, right))
                .collect();
            Ty::Enum(std::sync::Arc::new(enumeration))
        }
        (Ty::Record(left), Ty::Record(right)) => {
            let mut record = left.as_ref().clone();
            record.type_args = left
                .type_args
                .iter()
                .zip(&right.type_args)
                .map(|(left, right)| merge_inferred_types(left, right))
                .collect();
            for ((_, left), (_, right)) in record.fields.iter_mut().zip(&right.fields) {
                *left = merge_inferred_types(left, right);
            }
            Ty::Record(std::sync::Arc::new(record))
        }
        _ => left.clone(),
    }
}

/// Whether a candidate still contains unknown payload or element types.
pub(crate) fn has_inference_holes(ty: &Ty) -> bool {
    match ty {
        Ty::Error => true,
        Ty::Array(inner) | Ty::Channel(inner) | Ty::Option(inner) | Ty::Task(inner) => {
            has_inference_holes(inner)
        }
        Ty::Result(left, right) | Ty::Dict(left, right) => {
            has_inference_holes(left) || has_inference_holes(right)
        }
        Ty::Record(record) => record.type_args.iter().any(has_inference_holes),
        Ty::Enum(enumeration) => enumeration.type_args.iter().any(has_inference_holes),
        Ty::Function(function) => {
            function
                .params
                .iter()
                .any(|param| has_inference_holes(&param.ty))
                || has_inference_holes(&function.return_type)
        }
        Ty::Procedure(procedure) => procedure
            .params
            .iter()
            .any(|param| has_inference_holes(&param.ty)),
        _ => false,
    }
}

/// Whether an expected type can complete holes or instantiate a generic routine value.
pub(crate) fn needs_expected_type(ty: &Ty) -> bool {
    has_inference_holes(ty)
        || match ty {
            Ty::Function(function) => !function.type_params.is_empty(),
            Ty::Procedure(procedure) => !procedure.type_params.is_empty(),
            _ => false,
        }
}
