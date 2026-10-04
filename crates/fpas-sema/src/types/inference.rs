//! Complete inferred collection and constructor types from compatible value context.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::Ty;
use std::sync::Arc;

impl Ty {
    /// Return whether construction still contains unresolved empty-value type components.
    pub(crate) fn has_inference_holes(&self) -> bool {
        match self {
            Ty::Error => true,
            Ty::Array(inner) | Ty::Channel(inner) | Ty::Task(inner) | Ty::Option(inner) => {
                inner.has_inference_holes()
            }
            Ty::Result(left, right) | Ty::Dict(left, right) => {
                left.has_inference_holes() || right.has_inference_holes()
            }
            Ty::Applied(_, arguments) => arguments.iter().any(Ty::has_inference_holes),
            Ty::Record(record) => record.type_args.iter().any(Ty::has_inference_holes),
            Ty::Enum(enumeration) => enumeration.type_args.iter().any(Ty::has_inference_holes),
            Ty::Function(function) => {
                function
                    .params
                    .iter()
                    .any(|param| param.ty.has_inference_holes())
                    || function.return_type.has_inference_holes()
            }
            Ty::Procedure(procedure) => procedure
                .params
                .iter()
                .any(|param| param.ty.has_inference_holes()),
            _ => false,
        }
    }
    /// Fill absent type components from another already-compatible inferred type.
    ///
    /// Empty collections and the absent side of Option/Result constructors use `Error`
    /// placeholders. Preserve concrete types and generic parameter identities.
    pub(crate) fn complete_inference_with(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Applied(left, left_args), Self::Applied(right, right_args))
                if left.eq_ignore_ascii_case(right) && left_args.len() == right_args.len() =>
            {
                Self::Applied(
                    left.clone(),
                    left_args
                        .iter()
                        .zip(right_args)
                        .map(|(left, right)| left.complete_inference_with(right))
                        .collect(),
                )
            }
            (Self::Record(left), Self::Record(right))
                if left.name.eq_ignore_ascii_case(&right.name) =>
            {
                let mut record = left.as_ref().clone();
                record.type_args = left
                    .type_args
                    .iter()
                    .zip(&right.type_args)
                    .map(|(left, right)| left.complete_inference_with(right))
                    .collect();
                for ((_, field), (_, other)) in record.fields.iter_mut().zip(&right.fields) {
                    *field = field.complete_inference_with(other);
                }
                Self::Record(Arc::new(record))
            }
            (Self::Enum(left), Self::Enum(right))
                if left.name.eq_ignore_ascii_case(&right.name) =>
            {
                let mut enumeration = left.as_ref().clone();
                enumeration.type_args = left
                    .type_args
                    .iter()
                    .zip(&right.type_args)
                    .map(|(left, right)| left.complete_inference_with(right))
                    .collect();
                for (variant, other) in enumeration.variants.iter_mut().zip(&right.variants) {
                    for ((_, field), (_, other)) in variant.fields.iter_mut().zip(&other.fields) {
                        *field = field.complete_inference_with(other);
                    }
                }
                Self::Enum(Arc::new(enumeration))
            }
            (Self::Error, other) => other.clone(),
            (Self::Array(left), Self::Array(right)) => {
                Self::Array(Box::new(left.complete_inference_with(right)))
            }
            (Self::Option(left), Self::Option(right)) => {
                Self::Option(Box::new(left.complete_inference_with(right)))
            }
            (Self::Channel(left), Self::Channel(right)) => {
                Self::Channel(Box::new(left.complete_inference_with(right)))
            }
            (Self::Task(left), Self::Task(right)) => {
                Self::Task(Box::new(left.complete_inference_with(right)))
            }
            (Self::Dict(left_key, left_value), Self::Dict(right_key, right_value)) => Self::Dict(
                Box::new(left_key.complete_inference_with(right_key)),
                Box::new(left_value.complete_inference_with(right_value)),
            ),
            (Self::Result(left_ok, left_error), Self::Result(right_ok, right_error)) => {
                Self::Result(
                    Box::new(left_ok.complete_inference_with(right_ok)),
                    Box::new(left_error.complete_inference_with(right_error)),
                )
            }
            _ => self.clone(),
        }
    }
}
