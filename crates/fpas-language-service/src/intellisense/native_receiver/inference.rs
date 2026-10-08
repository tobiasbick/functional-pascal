//! Catalog result inference for syntactically recovered receiver chains.

use super::receiver_type;
use crate::navigation::NavigationDocument;
use fpas_parser::Expr;
use fpas_sema::Ty;

pub(super) fn shape(text: &str) -> Option<Ty> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    if lower == "string" {
        return Some(Ty::String);
    }
    for (prefix, constructor) in [
        ("array of ", Ty::Array as fn(Box<Ty>) -> Ty),
        ("option of ", Ty::Option),
    ] {
        if lower.starts_with(prefix) {
            return Some(constructor(Box::new(
                shape(&text[prefix.len()..])
                    .unwrap_or_else(|| Ty::Named(text[prefix.len()..].into())),
            )));
        }
    }
    if lower.starts_with("dict of ") {
        let split = lower.find(" to ")?;
        return Some(Ty::Dict(
            Box::new(shape(&text[8..split]).unwrap_or(Ty::Error)),
            Box::new(shape(&text[split + 4..]).unwrap_or(Ty::Error)),
        ));
    }
    if lower.starts_with("result of ") {
        let (ok, error) = text[10..].split_once(',')?;
        return Some(Ty::Result(
            Box::new(shape(ok).unwrap_or(Ty::Error)),
            Box::new(shape(error).unwrap_or(Ty::Error)),
        ));
    }
    match lower.as_str() {
        "integer" => Some(Ty::Integer),
        "boolean" => Some(Ty::Boolean),
        "real" => Some(Ty::Real),
        _ => None,
    }
}

pub(super) fn operation_result(
    documents: &[NavigationDocument],
    target: usize,
    receiver_ty: &Ty,
    name: &str,
    args: &[Expr],
    source: &str,
    offset: usize,
    depth: usize,
) -> Option<Ty> {
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
            Expr::Closure(closure) => shape(type_span(closure.return_type.as_ref()?).text(source)?),
            _ => receiver_type(
                documents,
                target,
                callback.span().text(source)?,
                offset,
                depth + 1,
            ),
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

fn type_span(ty: &fpas_parser::TypeExpr) -> fpas_lexer::Span {
    use fpas_parser::TypeExpr;
    match ty {
        TypeExpr::Named { span, .. }
        | TypeExpr::FunctionType { span, .. }
        | TypeExpr::ProcedureType { span, .. }
        | TypeExpr::Result { span, .. }
        | TypeExpr::Option { span, .. }
        | TypeExpr::Dict { span, .. }
        | TypeExpr::Array(_, span)
        | TypeExpr::Channel(_, span)
        | TypeExpr::Task(_, span) => *span,
    }
}
