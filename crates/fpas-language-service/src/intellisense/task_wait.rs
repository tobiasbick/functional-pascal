//! Result-aware signatures for the polymorphic task Wait operation.
//!
//! Documentation: `docs/pascal/language/concurrency/task-handles.md`.

use crate::CallableSignature;
use crate::navigation::NavigationDocument;
use fpas_lexer::Token;
use fpas_sema::Ty;

/// Shows a typed function result or the procedure form for the actual Wait argument.
pub(super) fn specialize(
    documents: &[NavigationDocument],
    target: usize,
    parenthesis: usize,
    offset: usize,
    qualified_name: &str,
    signature: &mut CallableSignature,
) {
    if !qualified_name.eq_ignore_ascii_case("Std.Tasks.Wait") {
        return;
    }
    let document = &documents[target];
    let Some(start) = document.tokens.get(parenthesis + 1) else {
        return;
    };
    let mut end = start.span.offset;
    let mut depth = 0usize;
    for token in document.tokens.iter().skip(parenthesis + 1) {
        if token.span.offset >= offset {
            break;
        }
        match token.token {
            Token::RParen | Token::RBracket if depth == 0 => break,
            Token::Comma if depth == 0 => break,
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth -= 1,
            _ => {}
        }
        end = (token.span.offset + token.span.length).min(offset);
    }
    let Some(argument) = document.snapshot.source().get(start.span.offset..end) else {
        return;
    };
    let argument = argument
        .split_once(":=")
        .map_or(argument, |(_, value)| value.trim());
    let Some(handle @ Ty::Task(_)) =
        super::native_receiver::receiver_type(documents, target, argument, offset, 0)
    else {
        return;
    };
    let Ty::Task(result) = &handle else {
        return;
    };
    signature.parameters = vec![format!("Handle: {handle}")];
    signature.label = if matches!(result.as_ref(), Ty::Unit) {
        format!("procedure Wait({})", signature.parameters[0])
    } else {
        format!("function Wait({}): {result}", signature.parameters[0])
    };
}
