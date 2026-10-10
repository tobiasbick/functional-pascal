//! Generic substitution for nested containers, records, and callable types.
//! See `docs/pascal/language/types/generics.md`.

use super::{MethodKind, Ty};
use std::collections::HashMap;
use std::sync::Arc;

/// Replace bound parameters without resolving recursive nominal references.
pub(crate) fn substitute_type_parameters(ty: &Ty, bindings: &HashMap<String, Ty>) -> Ty {
    if bindings.is_empty() {
        return ty.clone();
    }
    let substitute = |ty: &Ty| substitute_type_parameters(ty, bindings);
    match ty {
        Ty::GenericParam(name, _) => bindings
            .get(&name.to_ascii_lowercase())
            .cloned()
            .unwrap_or_else(|| ty.clone()),
        Ty::Array(inner) => Ty::Array(Box::new(substitute(inner))),
        Ty::Channel(inner) => Ty::Channel(Box::new(substitute(inner))),
        Ty::Task(inner) => Ty::Task(Box::new(substitute(inner))),
        Ty::Option(inner) => Ty::Option(Box::new(substitute(inner))),
        Ty::Result(ok, error) => Ty::Result(Box::new(substitute(ok)), Box::new(substitute(error))),
        Ty::Dict(key, value) => Ty::Dict(Box::new(substitute(key)), Box::new(substitute(value))),
        Ty::Function(function) => {
            let mut function = function.clone();
            for parameter in &mut function.params {
                parameter.ty = substitute(&parameter.ty);
            }
            function.return_type = Box::new(substitute(&function.return_type));
            Ty::Function(function)
        }
        Ty::Procedure(procedure) => {
            let mut procedure = procedure.clone();
            for parameter in &mut procedure.params {
                parameter.ty = substitute(&parameter.ty);
            }
            Ty::Procedure(procedure)
        }
        Ty::Enum(enumeration) => {
            if enumeration.type_params.is_empty() && enumeration.type_args.is_empty() {
                return ty.clone();
            }
            let mut enumeration = enumeration.as_ref().clone();
            enumeration.type_args = if enumeration.type_args.is_empty() {
                enumeration
                    .type_params
                    .iter()
                    .map(|parameter| {
                        substitute(&Ty::GenericParam(
                            parameter.name.clone(),
                            parameter.constraint,
                        ))
                    })
                    .collect()
            } else {
                enumeration.type_args.iter().map(substitute).collect()
            };
            for variant in &mut enumeration.variants {
                for (_, field) in &mut variant.fields {
                    *field = substitute(field);
                }
            }
            Ty::Enum(Arc::new(enumeration))
        }
        Ty::Record(record) => {
            if record.type_params.is_empty() && record.type_args.is_empty() {
                return ty.clone();
            }
            let mut record = record.as_ref().clone();
            if record.type_args.is_empty() && !record.type_params.is_empty() {
                record.type_args = record
                    .type_params
                    .iter()
                    .map(|parameter| {
                        substitute(&Ty::GenericParam(
                            parameter.name.clone(),
                            parameter.constraint,
                        ))
                    })
                    .collect();
            } else {
                record.type_args = record.type_args.iter().map(substitute).collect();
            }
            for (_, field) in &mut record.fields {
                *field = substitute(field);
            }
            for (_, method) in &mut record.methods {
                match method {
                    MethodKind::Function(function) => {
                        if let Ty::Function(value) = substitute(&Ty::Function(function.clone())) {
                            *function = value;
                        }
                    }
                    MethodKind::Procedure(procedure) => {
                        if let Ty::Procedure(value) = substitute(&Ty::Procedure(procedure.clone()))
                        {
                            *procedure = value;
                        }
                    }
                }
            }
            // Static members are accessed through the declaration, not a stored value.
            Ty::Record(Arc::new(record))
        }
        _ => ty.clone(),
    }
}
