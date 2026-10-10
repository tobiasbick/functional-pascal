//! Catalog result inference for syntactically recovered receiver chains.

use super::{receiver_type, types};
use crate::navigation::NavigationDocument;
use fpas_parser::Expr;
use fpas_sema::Ty;

/// Document context for nested receiver-type lookups.
pub(super) struct ReceiverLookup<'a> {
    pub(super) documents: &'a [NavigationDocument],
    pub(super) target: usize,
    pub(super) offset: usize,
    pub(super) depth: usize,
}

/// Infers catalog result types, including callback return types in recovered chains.
/// See `docs/pascal/tools/editor-integration.md`.
pub(super) fn operation_result(
    lookup: &ReceiverLookup<'_>,
    receiver_ty: &Ty,
    name: &str,
    args: &[Expr],
    source: &str,
) -> Option<Ty> {
    let ReceiverLookup {
        documents,
        target,
        offset,
        depth,
    } = *lookup;
    let operation = fpas_sema::native_operation(receiver_ty, name)?;
    let signature = operation.signature_for(Some(receiver_ty));
    let argument = |parameter: &str| {
        if args.iter().any(|arg| arg.argument_name().is_some()) {
            args.iter()
                .find(|arg| {
                    arg.argument_name()
                        .is_some_and(|name| name.eq_ignore_ascii_case(parameter))
                })
                .map(Expr::argument_value)
        } else {
            let index = signature
                .params
                .iter()
                .position(|param| param.name.eq_ignore_ascii_case(parameter))?;
            args.get(index)
        }
    };
    let inferred = if name.eq_ignore_ascii_case("Reduce") {
        let initial = argument("Init")?;
        receiver_type(
            documents,
            target,
            initial.span().text(source)?,
            offset,
            depth + 1,
        )
    } else if let Some(callback) = argument("F") {
        let result = match callback {
            Expr::Closure(closure) => {
                types::from_syntax(documents, target, closure.return_type.as_ref()?, depth + 1)
            }
            _ => receiver_type(
                documents,
                target,
                callback.span().text(source)?,
                offset,
                depth + 1,
            )
            .and_then(|ty| match ty {
                Ty::Function(function) => Some(*function.return_type),
                _ => None,
            }),
        };
        match (name.to_ascii_lowercase().as_str(), result) {
            ("flatmap", Some(Ty::Array(inner))) | ("andthen", Some(Ty::Option(inner))) => {
                Some(*inner)
            }
            ("andthen", Some(Ty::Result(ok, _))) => Some(*ok),
            ("orelse", Some(Ty::Result(_, error))) => Some(*error),
            (_, result) => result,
        }
    } else {
        None
    };
    Some(substitute_result(&signature.return_type, inferred.as_ref()))
}

fn substitute_result(ty: &Ty, inferred: Option<&Ty>) -> Ty {
    match ty {
        Ty::GenericParam(name, _)
            if ["U", "V2", "E2"]
                .iter()
                .any(|value| name.eq_ignore_ascii_case(value)) =>
        {
            inferred.cloned().unwrap_or_else(|| ty.clone())
        }
        Ty::Array(inner) => Ty::Array(Box::new(substitute_result(inner, inferred))),
        Ty::Option(inner) => Ty::Option(Box::new(substitute_result(inner, inferred))),
        Ty::Result(ok, error) => Ty::Result(
            Box::new(substitute_result(ok, inferred)),
            Box::new(substitute_result(error, inferred)),
        ),
        Ty::Dict(key, value) => Ty::Dict(key.clone(), Box::new(substitute_result(value, inferred))),
        _ => ty.clone(),
    }
}
