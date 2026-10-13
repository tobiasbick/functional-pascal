//! Payload and field roles inherited by an immutable pattern binding.

use crate::navigation::NavigationDocument;
use fpas_lexer::Token;
use fpas_parser::Expr;

/// One statically selected component of an aggregate receiving type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::navigation) enum Step {
    /// Array or dictionary value element.
    Element,
    /// Present Option payload.
    Some,
    /// Successful Result payload.
    Ok,
    /// Failed Result payload.
    Error,
    /// Result type of a callable value.
    Return,
    /// Declared enum payload field or record property.
    Field {
        variant: Option<String>,
        name: Option<String>,
        position: usize,
    },
}

/// Finds a case selector and the nested constructor/field path to a pattern binding.
pub(super) fn pattern_selector(
    document: &NavigationDocument,
    binding: usize,
) -> Option<(Expr, usize, Vec<Step>)> {
    let tokens = &document.tokens;
    let binding = tokens
        .iter()
        .position(|token| token.span.offset == binding)?;
    let mut cases = Vec::new();
    for index in 0..binding {
        if tokens[index].token == Token::Case {
            if index > 0 && tokens[index - 1].token == Token::End {
                cases.pop();
            } else {
                cases.push(index);
            }
        }
    }
    let case = *cases.last()?;
    let mut depth = 0usize;
    let mut nested_cases = 0usize;
    let of = (case + 1..binding).find(|&index| match tokens[index].token {
        Token::Case => {
            if tokens[index - 1].token == Token::End {
                nested_cases = nested_cases.saturating_sub(1);
            } else {
                nested_cases += 1;
            }
            false
        }
        Token::LParen | Token::LBracket => {
            depth += 1;
            false
        }
        Token::RParen | Token::RBracket => {
            depth = depth.saturating_sub(1);
            false
        }
        Token::Of if depth == 0 && nested_cases == 0 => true,
        _ => false,
    })?;
    let start = tokens.get(case + 1)?.span.offset;
    let (selector, errors) = fpas_parser::parse_expression(
        document
            .snapshot
            .source()
            .get(start..tokens[of].span.offset)?
            .trim(),
    );
    if !errors.is_empty() {
        return None;
    }
    let when = (of + 1..binding)
        .rev()
        .find(|&index| tokens[index].token == Token::When)?;
    let mut cursor = binding;
    let mut path = Vec::new();
    while cursor > when {
        let mut depth = 0usize;
        let open = (when + 1..cursor)
            .rev()
            .find(|&index| match tokens[index].token {
                Token::RParen => {
                    depth += 1;
                    false
                }
                Token::LParen if depth == 0 => true,
                Token::LParen => {
                    depth -= 1;
                    false
                }
                _ => false,
            });
        let Some(open) = open else {
            break;
        };
        let mut position = 0usize;
        let mut field_start = open + 1;
        let mut depth = 0usize;
        for index in open + 1..cursor {
            match tokens[index].token {
                Token::LParen | Token::LBracket => depth += 1,
                Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
                Token::Comma if depth == 0 => {
                    position += 1;
                    field_start = index + 1;
                }
                _ => {}
            }
        }
        let label = if tokens.get(field_start + 1)?.token == Token::ColonAssign {
            crate::navigation::token_name(document, field_start)
        } else {
            None
        };
        path.push(match &tokens[open - 1].token {
            Token::Some => Step::Some,
            Token::Ok => Step::Ok,
            Token::Error => Step::Error,
            Token::Ident(variant) => Step::Field {
                variant: Some(variant.clone()),
                name: label,
                position,
            },
            _ => return None,
        });
        cursor = open;
    }
    path.reverse();
    Some((selector, start, path))
}
