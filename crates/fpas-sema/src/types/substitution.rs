//! Shared substitution for generic routines and nominal data applications.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use std::sync::Arc;

use super::{GenericParamDef, Ty, TypeArguments};

impl Ty {
    /// Return the declared parameters of an unapplied nominal generic type.
    pub(crate) fn type_parameters(&self) -> &[GenericParamDef] {
        match self {
            Ty::Record(record) if record.type_args.is_empty() => &record.type_params,
            Ty::Enum(enumeration) if enumeration.type_args.is_empty() => &enumeration.type_params,
            _ => &[],
        }
    }

    /// Instantiate a nominal template while retaining its declaration identity.
    pub(crate) fn instantiate(&self, arguments: &[Ty]) -> Option<Ty> {
        let parameters = self.type_parameters();
        if parameters.len() != arguments.len() {
            return None;
        }
        let bindings = parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.identity.clone(), argument.clone()))
            .collect();
        let mut instantiated = self.substitute(&bindings);
        match &mut instantiated {
            Ty::Record(record) => Arc::make_mut(record).type_args = arguments.to_vec(),
            Ty::Enum(enumeration) => Arc::make_mut(enumeration).type_args = arguments.to_vec(),
            _ => return None,
        }
        Some(instantiated)
    }

    /// Replace parameters throughout a type, leaving recursive references deferred.
    pub(crate) fn substitute(&self, bindings: &TypeArguments) -> Ty {
        let substitute = |ty: &Ty| ty.substitute(bindings);
        match self {
            Ty::GenericParam(parameter) => bindings
                .get(&parameter.identity)
                .cloned()
                .unwrap_or_else(|| self.clone()),
            Ty::Applied(name, arguments) => {
                Ty::Applied(name.clone(), arguments.iter().map(substitute).collect())
            }
            Ty::Array(inner) => Ty::Array(Box::new(substitute(inner))),
            Ty::Channel(inner) => Ty::Channel(Box::new(substitute(inner))),
            Ty::Task(inner) => Ty::Task(Box::new(substitute(inner))),
            Ty::Option(inner) => Ty::Option(Box::new(substitute(inner))),
            Ty::Result(ok, error) => {
                Ty::Result(Box::new(substitute(ok)), Box::new(substitute(error)))
            }
            Ty::Dict(key, value) => {
                Ty::Dict(Box::new(substitute(key)), Box::new(substitute(value)))
            }
            Ty::Function(function) => {
                let mut substituted = function.clone();
                for parameter in &mut substituted.params {
                    parameter.ty = substitute(&parameter.ty);
                }
                substituted.return_type = Box::new(substitute(&substituted.return_type));
                Ty::Function(substituted)
            }
            Ty::Procedure(procedure) => {
                let mut substituted = procedure.clone();
                for parameter in &mut substituted.params {
                    parameter.ty = substitute(&parameter.ty);
                }
                Ty::Procedure(substituted)
            }
            Ty::Record(record) => {
                let mut substituted = record.as_ref().clone();
                substituted.type_args = record.type_args.iter().map(substitute).collect();
                for (_, field) in &mut substituted.fields {
                    *field = substitute(field);
                }
                Ty::Record(Arc::new(substituted))
            }
            Ty::Enum(enumeration) => {
                let mut substituted = enumeration.as_ref().clone();
                substituted.type_args = enumeration.type_args.iter().map(substitute).collect();
                for variant in &mut substituted.variants {
                    for (_, field) in &mut variant.fields {
                        *field = substitute(field);
                    }
                }
                Ty::Enum(Arc::new(substituted))
            }
            _ => self.clone(),
        }
    }
}
