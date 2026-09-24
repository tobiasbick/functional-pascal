//! Callable resolution and active-argument tracking for nested source calls.

use std::path::Path;

use fpas_lexer::Token;

use super::SignatureHelp;
use crate::navigation::{NavigationDocument, NavigationResult, resolve, token_name};
use crate::{
    LanguageService, LanguageServiceError,
    documentation::{parameter_documentation, preceding_documentation},
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
            let (document_index, symbol, _) =
                resolve(&context.documents, target_index, callable_token.span.offset)?;
            let mut signature = symbol.callable?;
            let is_receiver_call = document
                .analysis
                .as_ref()
                .and_then(|analysis| analysis.semantic())
                .is_some_and(|semantic| {
                    semantic.metadata().fluent_calls.values().any(|call| {
                        call.name
                            .rsplit('.')
                            .next()
                            .is_some_and(|short| short.eq_ignore_ascii_case(&symbol.name))
                            && call.call_span.offset <= callable_token.span.offset
                            && callable_token.span.offset
                                < call.call_span.offset.saturating_add(call.call_span.length)
                    })
                });
            if is_receiver_call && !signature.parameters.is_empty() {
                signature.parameters.remove(0);
                if let Some(open) = signature.label.find('(')
                    && let Some(close) = signature.label.rfind(')')
                    && open < close
                {
                    signature
                        .label
                        .replace_range(open + 1..close, &signature.parameters.join("; "));
                }
            }
            let documentation = preceding_documentation(
                context.documents[document_index].snapshot.source(),
                symbol.full_span.offset(),
            );
            let mut parameter_documentation = documentation.as_deref().map_or_else(
                || vec![None; signature.parameters.len()],
                |documentation| parameter_documentation(documentation, &signature.parameters),
            );
            parameter_documentation.truncate(signature.parameters.len());
            let active_parameter = (!signature.parameters.is_empty())
                .then(|| frame.active_argument.min(signature.parameters.len() - 1));
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

#[derive(Debug, Clone, Copy)]
struct CallFrame {
    callable_token: usize,
    active_argument: usize,
}

#[derive(Debug, Clone, Copy)]
enum Delimiter {
    Parenthesis(Option<CallFrame>),
    Bracket,
}

fn active_call(document: &NavigationDocument, offset: usize) -> Option<CallFrame> {
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
                    active_argument: 0,
                }),
            )),
            Token::RParen => pop_parenthesis(&mut delimiters),
            Token::LBracket => delimiters.push(Delimiter::Bracket),
            Token::RBracket => pop_bracket(&mut delimiters),
            Token::Comma => {
                if let Some(Delimiter::Parenthesis(Some(frame))) = delimiters.last_mut() {
                    frame.active_argument = frame.active_argument.saturating_add(1);
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
    parenthesis
        .checked_sub(1)
        .filter(|index| token_name(document, *index).is_some())
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
