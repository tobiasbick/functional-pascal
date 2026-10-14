//! Callable resolution and active-argument tracking for nested source calls.

use std::path::Path;

use fpas_lexer::Token;

use super::SignatureHelp;
use crate::navigation::{NavigationDocument, NavigationResult, resolve, token_name};
use crate::{
    LanguageService, LanguageServiceError,
    documentation::{parameter_documentation, parameter_name, preceding_documentation},
};

impl LanguageService {
    /// Returns signature help for the innermost call containing a UTF-8 byte offset.
    pub fn signature_help(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Option<SignatureHelp>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context.target_index.and_then(|target_index| {
            let document = &context.documents[target_index];
            let frame = active_call(document, offset)?;
            let callable_token = &document.tokens[frame.callable_token];
            if let Some(signature) = payload_signature(&callable_token.token) {
                return Some(SignatureHelp {
                    parameter_documentation: vec![None],
                    signature,
                    documentation: None,
                    active_parameter: Some(0),
                });
            }
            if let Some((entry, receiver)) = super::native::native_at(
                &context.documents,
                target_index,
                callable_token.span.offset,
            ) {
                let signature = super::native::signature(entry, receiver.as_ref());
                let active_parameter = frame
                    .argument_name
                    .and_then(|token| token_name(document, token))
                    .and_then(|name| {
                        signature.parameters.iter().position(|parameter| {
                            parameter_name(parameter)
                                .is_some_and(|declared| declared.eq_ignore_ascii_case(&name))
                        })
                    })
                    .or_else(|| {
                        (!signature.parameters.is_empty())
                            .then(|| frame.active_argument.min(signature.parameters.len() - 1))
                    });
                return Some(SignatureHelp {
                    parameter_documentation: vec![None; signature.parameters.len()],
                    signature,
                    documentation: Some(entry.documentation.into()),
                    active_parameter,
                });
            }
            let Some((mut document_index, mut symbol, _)) =
                resolve(&context.documents, target_index, callable_token.span.offset)
            else {
                let signature = super::callable_values::signature(
                    &context.documents,
                    target_index,
                    frame.callable_token,
                    frame.parenthesis,
                    offset,
                )?;
                let active_parameter = frame
                    .argument_name
                    .and_then(|token| token_name(document, token))
                    .and_then(|name| {
                        signature.parameters.iter().position(|parameter| {
                            parameter_name(parameter)
                                .is_some_and(|declared| declared.eq_ignore_ascii_case(&name))
                        })
                    })
                    .or_else(|| {
                        (!signature.parameters.is_empty())
                            .then(|| frame.active_argument.min(signature.parameters.len() - 1))
                    });
                return Some(SignatureHelp {
                    parameter_documentation: vec![None; signature.parameters.len()],
                    signature,
                    documentation: None,
                    active_parameter,
                });
            };
            let called_name = symbol.name.clone();
            let static_signature = (!matches!(
                symbol.kind,
                crate::SymbolKind::Function
                    | crate::SymbolKind::Procedure
                    | crate::SymbolKind::Method
            ))
            .then(|| {
                super::callable_values::signature(
                    &context.documents,
                    target_index,
                    frame.callable_token,
                    frame.parenthesis,
                    offset,
                )
            })
            .flatten();
            if symbol.callable.is_none()
                && let Some(owner) = crate::navigation::named_arguments::callable_type_owner(
                    &context.documents,
                    target_index,
                    document_index,
                    &symbol,
                )
            {
                (document_index, symbol) = owner;
            }
            let qualified_name = symbol.qualified_name.clone();
            let mut signature = if symbol.kind == crate::SymbolKind::Type
                && symbol
                    .children
                    .iter()
                    .any(|child| child.kind == crate::SymbolKind::Field)
            {
                let (_, record) = crate::navigation::record_construction::constructor_record(
                    &context.documents,
                    target_index,
                    document_index,
                    symbol.clone(),
                )?;
                let mut signature = record.callable?;
                signature.label = format!(
                    "{}({}): {}",
                    called_name,
                    signature.parameters.join("; "),
                    called_name
                );
                signature
            } else {
                static_signature.or(symbol.callable)?
            };
            super::task_wait::specialize(
                &context.documents,
                target_index,
                frame.parenthesis,
                offset,
                &qualified_name,
                &mut signature,
            );
            let documentation = preceding_documentation(
                context.documents[document_index].snapshot.source(),
                symbol.full_span.offset(),
            );
            let mut parameter_documentation = documentation.as_deref().map_or_else(
                || vec![None; signature.parameters.len()],
                |documentation| parameter_documentation(documentation, &signature.parameters),
            );
            parameter_documentation.truncate(signature.parameters.len());
            let named_parameter = frame
                .argument_name
                .and_then(|token| token_name(document, token))
                .and_then(|name| {
                    signature.parameters.iter().position(|parameter| {
                        parameter_name(parameter)
                            .is_some_and(|declared| declared.eq_ignore_ascii_case(&name))
                    })
                });
            let active_parameter = named_parameter.or_else(|| {
                (!signature.parameters.is_empty())
                    .then(|| frame.active_argument.min(signature.parameters.len() - 1))
            });
            Some(SignatureHelp {
                signature,
                documentation,
                parameter_documentation,
                active_parameter,
            })
        });
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }
}

/// Describes the canonical payload label of built-in construction and patterns.
pub(super) fn payload_signature(token: &Token) -> Option<crate::CallableSignature> {
    let (name, result) = match token {
        Token::Some => ("Some", "option of T"),
        Token::Ok => ("Ok", "result of (T, E)"),
        Token::Error => ("Error", "result of (T, E)"),
        _ => return None,
    };
    let payload = if matches!(token, Token::Error) {
        "E"
    } else {
        "T"
    };
    Some(crate::CallableSignature {
        label: format!("{name}(Value: {payload}): {result}"),
        parameters: vec![format!("Value: {payload}")],
    })
}

#[derive(Debug, Clone, Copy)]
/// Innermost call and current argument boundaries shared by help and field completion.
pub(super) struct CallFrame {
    pub(super) callable_token: usize,
    pub(super) parenthesis: usize,
    pub(super) argument_start: usize,
    active_argument: usize,
    /// Name token of the current `Name := Value` argument.
    argument_name: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
enum Delimiter {
    Parenthesis(Option<CallFrame>),
    Bracket,
}

/// Track nested delimiters to identify the active source call at a byte offset.
pub(super) fn active_call(document: &NavigationDocument, offset: usize) -> Option<CallFrame> {
    let tokens = &document.tokens;
    let mut delimiters = Vec::<Delimiter>::new();
    for (index, token) in tokens
        .iter()
        .enumerate()
        .take_while(|(_, token)| token.span.offset < offset)
    {
        match token.token {
            Token::LParen => delimiters.push(Delimiter::Parenthesis(
                callable_before(document, index).map(|callable_token| CallFrame {
                    callable_token,
                    parenthesis: index,
                    argument_start: index + 1,
                    active_argument: 0,
                    argument_name: None,
                }),
            )),
            Token::RParen => pop_parenthesis(&mut delimiters),
            Token::LBracket => delimiters.push(Delimiter::Bracket),
            Token::RBracket => pop_bracket(&mut delimiters),
            Token::Comma => {
                if let Some(Delimiter::Parenthesis(Some(frame))) = delimiters.last_mut() {
                    frame.active_argument = frame.active_argument.saturating_add(1);
                    frame.argument_start = index + 1;
                    frame.argument_name = None;
                }
            }
            Token::ColonAssign => {
                if let Some(Delimiter::Parenthesis(Some(frame))) = delimiters.last_mut()
                    && let Some(name) = index.checked_sub(1)
                    && matches!(tokens[name].token, Token::Ident(_))
                    && name.checked_sub(1).is_some_and(|start| {
                        matches!(tokens[start].token, Token::LParen | Token::Comma)
                    })
                {
                    frame.argument_name = Some(name);
                }
            }
            _ => {}
        }
    }
    delimiters
        .iter()
        .rev()
        .find_map(|delimiter| match delimiter {
            Delimiter::Parenthesis(Some(frame)) => Some(*frame),
            Delimiter::Parenthesis(None) | Delimiter::Bracket => None,
        })
}

fn callable_before(document: &NavigationDocument, parenthesis: usize) -> Option<usize> {
    crate::navigation::callable_labels::callable_before(document, parenthesis)
}

fn pop_parenthesis(delimiters: &mut Vec<Delimiter>) {
    while let Some(delimiter) = delimiters.pop() {
        if matches!(delimiter, Delimiter::Parenthesis(_)) {
            break;
        }
    }
}

fn pop_bracket(delimiters: &mut Vec<Delimiter>) {
    while let Some(delimiter) = delimiters.pop() {
        if matches!(delimiter, Delimiter::Bracket) {
            break;
        }
    }
}
