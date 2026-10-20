//! Alias-free operational type identities for native errors.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::{GenericParamDef, Ty};
use fpas_unit::errors::{IdentityParameter, NominalKind, TypeIdentity};
mod schema;
pub use schema::error_schema;

/// Convert a checked semantic type without expanding nominal fields.
///
/// # Errors
/// Rejects unresolved parameters, missing nominal definitions and excessive depth.
pub fn semantic_type_identity(
    ty: &Ty,
    owner: &str,
    parameters: &[GenericParamDef],
    resolve: &impl Fn(&str) -> Option<Ty>,
) -> Result<TypeIdentity, String> {
    fn visit(
        ty: &Ty,
        owner: &str,
        parameters: &[GenericParamDef],
        resolve: &impl Fn(&str) -> Option<Ty>,
        depth: usize,
    ) -> Result<TypeIdentity, String> {
        if depth > 64 {
            return Err("error type identity nesting exceeds limit".into());
        }
        let child = |ty: &Ty| visit(ty, owner, parameters, resolve, depth + 1);
        let nominal = |kind, name: &str, arguments: &[Ty], declared: &[GenericParamDef]| {
            let arguments = if arguments.is_empty() && !declared.is_empty() {
                declared
                    .iter()
                    .map(|parameter| {
                        child(&Ty::GenericParam(
                            parameter.name.clone(),
                            parameter.constraint,
                        ))
                    })
                    .collect::<Result<_, _>>()?
            } else {
                arguments.iter().map(child).collect::<Result<_, _>>()?
            };
            Ok::<_, String>(TypeIdentity::Nominal {
                kind,
                declaration: qualified_name(name, owner),
                arguments,
            })
        };
        Ok(match ty {
            Ty::Unit => TypeIdentity::Unit,
            Ty::Integer => TypeIdentity::Integer,
            Ty::Subrange(range) => TypeIdentity::Subrange {
                lower: range.lower,
                upper: range.upper,
            },
            Ty::Real => TypeIdentity::Real,
            Ty::Boolean => TypeIdentity::Boolean,
            Ty::String => TypeIdentity::String,
            Ty::Array(inner) => TypeIdentity::Array(Box::new(child(inner)?)),
            Ty::Channel(inner) => TypeIdentity::Channel(Box::new(child(inner)?)),
            Ty::Task(inner) => TypeIdentity::Task(Box::new(child(inner)?)),
            Ty::Option(inner) => TypeIdentity::Option(Box::new(child(inner)?)),
            Ty::Result(ok, error) => {
                TypeIdentity::Result(Box::new(child(ok)?), Box::new(child(error)?))
            }
            Ty::Dict(key, value) => {
                TypeIdentity::Dictionary(Box::new(child(key)?), Box::new(child(value)?))
            }
            Ty::Record(record) => nominal(
                NominalKind::Record,
                &qualified_name(&record.name, record.owner_unit.as_deref().unwrap_or(owner)),
                &record.type_args,
                &record.type_params,
            )?,
            Ty::Enum(enumeration) => nominal(
                if enumeration.is_error {
                    NominalKind::Error
                } else if enumeration.has_data() {
                    NominalKind::Enum
                } else {
                    NominalKind::OrdinalEnum
                },
                &enumeration.name,
                &enumeration.type_args,
                &enumeration.type_params,
            )?,
            Ty::Distinct(distinct) => TypeIdentity::Nominal {
                kind: NominalKind::Distinct,
                declaration: qualified_name(
                    &distinct.name,
                    distinct.owner_unit.as_deref().unwrap_or(owner),
                ),
                arguments: vec![],
            },
            Ty::GenericParam(name, _) => TypeIdentity::Parameter(
                u16::try_from(
                    parameter_index(parameters, name)
                        .ok_or_else(|| format!("unknown error identity parameter {name}"))?,
                )
                .map_err(|_| "too many error identity parameters")?,
            ),
            Ty::Named(name) => {
                let resolved = resolve(name)
                    .ok_or_else(|| format!("unresolved error identity type {name}"))?;
                if matches!(resolved, Ty::Named(_)) {
                    return Err(format!("unresolved error identity type {name}"));
                }
                child(&resolved)?
            }
            Ty::Function(function) => TypeIdentity::Callable {
                parameters: function
                    .params
                    .iter()
                    .map(|parameter| {
                        Ok(IdentityParameter {
                            name: parameter.name.to_ascii_lowercase(),
                            mutable: parameter.mode == super::ParamMode::Var,
                            ty: child(&parameter.ty)?,
                        })
                    })
                    .collect::<Result<_, String>>()?,
                result: Some(Box::new(child(&function.return_type)?)),
                variadic: function.variadic,
            },
            Ty::Procedure(procedure) => TypeIdentity::Callable {
                parameters: procedure
                    .params
                    .iter()
                    .map(|parameter| {
                        Ok(IdentityParameter {
                            name: parameter.name.to_ascii_lowercase(),
                            mutable: parameter.mode == super::ParamMode::Var,
                            ty: child(&parameter.ty)?,
                        })
                    })
                    .collect::<Result<_, String>>()?,
                result: None,
                variadic: procedure.variadic,
            },
            Ty::Error => return Err("incomplete error identity type".into()),
        })
    }
    visit(ty, owner, parameters, resolve, 0)
}

fn qualified_name(name: &str, owner: &str) -> String {
    if name.contains('.') {
        name.to_ascii_lowercase()
    } else {
        format!("{owner}.{name}").to_ascii_lowercase()
    }
}

fn parameter_index(parameters: &[GenericParamDef], name: &str) -> Option<usize> {
    parameters
        .iter()
        .position(|parameter| parameter.name.eq_ignore_ascii_case(name))
        .or_else(|| {
            let short = name.rsplit('.').next()?;
            let mut matches = parameters.iter().enumerate().filter(|(_, parameter)| {
                parameter
                    .name
                    .rsplit('.')
                    .next()
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(short))
            });
            let (index, _) = matches.next()?;
            if matches.next().is_some() {
                None
            } else {
                Some(index)
            }
        })
}
