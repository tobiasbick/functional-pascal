//! Callable resolution and active-argument tracking for nested source calls.

use std::path::Path;

use fpas_lexer::{SpannedToken, Token};

use super::SignatureHelp;
use crate::navigation::{NavigationResult, resolve};
use crate::{LanguageService, LanguageServiceError};

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
            let frame = active_call(&document.tokens, offset)?;
            let callable_token = &document.tokens[frame.callable_token];
            let (_, symbol, _) =
                resolve(&context.documents, target_index, callable_token.span.offset)?;
            let signature = symbol.callable?;
            let active_parameter = (!signature.parameters.is_empty())
                .then(|| frame.active_argument.min(signature.parameters.len() - 1));
            Some(SignatureHelp {
                signature,
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

fn active_call(tokens: &[SpannedToken], offset: usize) -> Option<CallFrame> {
    let mut delimiters = Vec::<Delimiter>::new();
    for (index, token) in tokens
        .iter()
        .enumerate()
        .take_while(|(_, token)| token.span.offset < offset)
    {
        match token.token {
            Token::LParen => delimiters.push(Delimiter::Parenthesis(
                callable_before(tokens, index).map(|callable_token| CallFrame {
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

fn callable_before(tokens: &[SpannedToken], parenthesis: usize) -> Option<usize> {
    parenthesis
        .checked_sub(1)
        .filter(|index| matches!(tokens[*index].token, Token::Ident(_)))
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
