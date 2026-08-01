//! Source spans and compact declaration text.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::{Span, Token, lex};
use fpas_parser::{FormalParam, FunctionDecl, ProcedureDecl, TypeExpr};

use crate::{CallableSignature, DocumentSnapshot};

pub(crate) fn name_span(
    snapshot: &DocumentSnapshot,
    full_span: SourceSpan,
    name: &str,
) -> SourceSpan {
    let source = snapshot.source();
    let end = full_span
        .offset
        .saturating_add(full_span.length)
        .min(source.len());
    let Some(fragment) = source.get(full_span.offset..end) else {
        return empty_span(full_span);
    };
    let (tokens, _) = lex(fragment);
    let Some(token) = tokens.into_iter().find(
        |token| matches!(&token.token, Token::Ident(value) if value.eq_ignore_ascii_case(name)),
    ) else {
        return empty_span(full_span);
    };
    let offset = full_span.offset + token.span.offset;
    let Some(position) = snapshot.line_index().position(source, offset) else {
        return empty_span(full_span);
    };
    SourceSpan::new_with_source(
        offset,
        token.span.length,
        u32::try_from(position.line + 1).unwrap_or(u32::MAX),
        u32::try_from(position.byte_column + 1).unwrap_or(u32::MAX),
        full_span.source_id,
    )
}

pub(crate) fn function_detail(snapshot: &DocumentSnapshot, value: &FunctionDecl) -> String {
    function_signature(snapshot, value, 0).label
}

pub(crate) fn procedure_detail(snapshot: &DocumentSnapshot, value: &ProcedureDecl) -> String {
    procedure_signature(snapshot, value, 0).label
}

pub(crate) fn function_signature(
    snapshot: &DocumentSnapshot,
    value: &FunctionDecl,
    implicit_parameters: usize,
) -> CallableSignature {
    let parameters = parameter_labels(snapshot, &value.params, implicit_parameters);
    CallableSignature {
        label: format!(
            "function {}{}({}): {}",
            value.name,
            type_parameters(&value.type_params),
            parameters.join("; "),
            type_text(snapshot, &value.return_type)
        ),
        parameters,
    }
}

pub(crate) fn procedure_signature(
    snapshot: &DocumentSnapshot,
    value: &ProcedureDecl,
    implicit_parameters: usize,
) -> CallableSignature {
    let parameters = parameter_labels(snapshot, &value.params, implicit_parameters);
    CallableSignature {
        label: format!(
            "procedure {}{}({})",
            value.name,
            type_parameters(&value.type_params),
            parameters.join("; ")
        ),
        parameters,
    }
}

pub(crate) fn type_callable_signature(
    snapshot: &DocumentSnapshot,
    name: &str,
    value: &TypeExpr,
) -> Option<CallableSignature> {
    match value {
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => {
            let parameters = parameter_labels(snapshot, params, 0);
            Some(CallableSignature {
                label: format!(
                    "function {name}({}): {}",
                    parameters.join("; "),
                    type_text(snapshot, return_type)
                ),
                parameters,
            })
        }
        TypeExpr::ProcedureType { params, .. } => {
            let parameters = parameter_labels(snapshot, params, 0);
            Some(CallableSignature {
                label: format!("procedure {name}({})", parameters.join("; ")),
                parameters,
            })
        }
        _ => None,
    }
}

pub(crate) fn named_type(value: &TypeExpr) -> Option<String> {
    match value {
        TypeExpr::Named { id, .. } => Some(id.parts.join(".")),
        _ => None,
    }
}

pub(crate) fn type_text(snapshot: &DocumentSnapshot, value: &TypeExpr) -> String {
    let span = type_span(value);
    snapshot
        .source()
        .get(span.offset..span.offset.saturating_add(span.length))
        .unwrap_or("<type>")
        .trim()
        .to_owned()
}

fn parameter_labels(
    snapshot: &DocumentSnapshot,
    params: &[FormalParam],
    implicit_parameters: usize,
) -> Vec<String> {
    params
        .iter()
        .skip(implicit_parameters)
        .map(|param| {
            format!(
                "{}{}: {}",
                if param.mutable { "mutable " } else { "" },
                param.name,
                type_text(snapshot, &param.type_expr)
            )
        })
        .collect()
}

fn type_parameters(parameters: &[fpas_parser::TypeParam]) -> String {
    if parameters.is_empty() {
        return String::new();
    }
    let values = parameters
        .iter()
        .map(|parameter| {
            parameter.constraint.as_ref().map_or_else(
                || parameter.name.clone(),
                |constraint| format!("{}: {constraint}", parameter.name),
            )
        })
        .collect::<Vec<_>>();
    format!("<{}>", values.join(", "))
}

fn empty_span(full_span: SourceSpan) -> SourceSpan {
    SourceSpan::new_with_source(
        full_span.offset,
        0,
        full_span.line,
        full_span.column,
        full_span.source_id,
    )
}

fn type_span(value: &TypeExpr) -> Span {
    match value {
        TypeExpr::Named { span, .. }
        | TypeExpr::Array(_, span)
        | TypeExpr::FunctionType { span, .. }
        | TypeExpr::ProcedureType { span, .. }
        | TypeExpr::Result { span, .. }
        | TypeExpr::Option { span, .. }
        | TypeExpr::Dict { span, .. } => *span,
    }
}
