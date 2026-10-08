//! Catalog-backed completion, signatures, and hover for built-in type operations.

use super::native_receiver::receiver_type;
use super::{CompletionCandidate, CompletionKind, CompletionSource};
use crate::navigation::{NavigationDocument, token_name};
use crate::{CallableSignature, SymbolKind};
use fpas_sema::{
    NativeOperation, Ty, native_factory, native_operation_by_implementation, native_operations,
};

/// Finds a resolved native operation at its source name, independent of imports.
pub(crate) fn native_at(
    documents: &[NavigationDocument],
    target: usize,
    offset: usize,
) -> Option<(&'static NativeOperation, Option<Ty>)> {
    let document = &documents[target];
    let (index, token) = document.tokens.iter().enumerate().find(|(_, token)| {
        token.span.offset <= offset && offset < token.span.offset + token.span.length
    })?;
    let name = token_name(document, index)?;
    if let Some(semantic) = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
    {
        let entry = semantic
            .metadata()
            .fluent_calls
            .values()
            .filter_map(|call| {
                let operation = native_operation_by_implementation(&call.name)?;
                if !operation.name.eq_ignore_ascii_case(&name)
                    || !(call.call_span.offset <= offset
                        && offset < call.call_span.offset + call.call_span.length)
                {
                    return None;
                }
                let first = document.tokens.iter().enumerate().find(|(i, token)| {
                    token.span.offset >= call.call_span.offset
                        && token_name(document, *i)
                            .is_some_and(|n| n.eq_ignore_ascii_case(operation.name))
                })?;
                (first.1.span.offset == token.span.offset).then_some((call, operation))
            })
            .min_by_key(|(call, _)| call.call_span.length);
        if let Some((call, operation)) = entry {
            return Some((operation, Some(call.receiver_ty.clone())));
        }
    }
    if index >= 2 && matches!(document.tokens[index - 1].token, fpas_lexer::Token::Dot) {
        let owner = match document.tokens[index - 2].token {
            fpas_lexer::Token::Array => Some("array".into()),
            _ => token_name(document, index - 2),
        };
        if let Some(entry) = owner.and_then(|owner| native_factory(&owner, &name)) {
            return Some((entry, None));
        }
    }
    let receiver = super::context::receiver_before(document.snapshot.source(), token.span.offset)?;
    let ty = receiver_type(documents, target, &receiver, offset, 0)?;
    fpas_sema::native_operation(&ty, &name).map(|entry| (entry, Some(ty)))
}

/// Public explicit signature; Format exposes its positional heterogeneous tail.
pub(crate) fn signature(entry: &NativeOperation, receiver: Option<&Ty>) -> CallableSignature {
    let ty = entry.signature_for(receiver);
    let mut parameters = ty
        .params
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if ty.variadic {
        parameters.push("Arguments...".into());
    }
    let result = if *ty.return_type == Ty::Unit {
        String::new()
    } else {
        format!(": {}", ty.return_type)
    };
    CallableSignature {
        label: format!(
            "{}.{}({}){result}",
            entry.receiver.label(),
            entry.name,
            parameters.join("; ")
        ),
        parameters,
    }
}

/// Produces catalog entries for a value receiver or one of the keyword factory owners.
pub(super) fn completions(
    documents: &[NavigationDocument],
    target: usize,
    receiver: &str,
    offset: usize,
    replacement: fpas_diagnostics::SourceSpan,
) -> Vec<CompletionCandidate> {
    let ty = receiver_type(documents, target, receiver, offset, 0);
    native_operations()
        .filter(|entry| {
            ty.as_ref().is_some_and(|ty| entry.receiver.accepts(ty))
                || matches!(
                    entry.receiver,
                    fpas_sema::NativeReceiver::StringFactory
                        | fpas_sema::NativeReceiver::ArrayFactory
                ) && entry.receiver.label().eq_ignore_ascii_case(receiver)
        })
        .map(|entry| CompletionCandidate {
            label: entry.name.into(),
            kind: CompletionKind::Symbol(
                if *entry.signature_for(ty.as_ref()).return_type == Ty::Unit {
                    SymbolKind::Procedure
                } else {
                    SymbolKind::Function
                },
            ),
            detail: signature(entry, ty.as_ref()).label,
            owner: Some(entry.receiver.label().into()),
            qualified_name: format!("{}.{}", entry.receiver.label(), entry.name),
            sort_text: format!("00:{}", entry.name.to_ascii_lowercase()),
            filter_text: entry.name.into(),
            insert_text: entry.name.into(),
            replacement_span: replacement,
            source: CompletionSource::Declaration,
            inline_documentation: Some(entry.documentation.into()),
            documentation: None,
            additional_edit: None,
        })
        .collect()
}
