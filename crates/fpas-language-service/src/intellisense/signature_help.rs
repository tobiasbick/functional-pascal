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
            let signature = symbol.callable?;
            let documentation = preceding_documentation(
                context.documents[document_index].snapshot.source(),
                symbol.full_span.offset(),
            );
            let parameter_documentation = documentation.as_deref().map_or_else(
                || vec![None; signature.parameters.len()],
                |documentation| parameter_documentation(documentation, &signature.parameters),
            );
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
