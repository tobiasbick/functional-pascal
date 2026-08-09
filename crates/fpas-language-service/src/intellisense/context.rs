//! Lexical completion context without interpreting text inside comments or strings.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Token;

use crate::navigation::NavigationDocument;

#[derive(Debug)]
pub(super) struct CompletionContext {
    pub(super) receiver: Option<String>,
    pub(super) prefix: String,
    pub(super) replacement: SourceSpan,
    pub(super) statements: bool,
}

pub(super) fn completion_context(
    document: &NavigationDocument,
    offset: usize,
) -> Option<CompletionContext> {
    let source = document.snapshot.source();
    let offset = offset.min(source.len());
    if !is_code_offset(source, offset) {
        return None;
    }
    let start = identifier_start(source, offset);
    let end = identifier_end(source, offset);
    let prefix = source.get(start..offset)?.to_owned();
    let receiver = receiver_before(source, start);
    Some(CompletionContext {
        receiver,
        prefix,
        replacement: SourceSpan::new(start, end.saturating_sub(start), 1, 1),
        statements: statement_context(document, offset),
    })
}

fn identifier_start(source: &str, offset: usize) -> usize {
    let bytes = source.as_bytes();
    let mut start = offset;
    while start > 0 && identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    start
}

fn identifier_end(source: &str, offset: usize) -> usize {
    let bytes = source.as_bytes();
    let mut end = offset;
    while end < bytes.len() && identifier_byte(bytes[end]) {
        end += 1;
    }
    end
}

fn receiver_before(source: &str, replacement_start: usize) -> Option<String> {
    let before = source.get(..replacement_start)?;
    let dot = before.strip_suffix('.')?;
    let start = dot
        .char_indices()
        .rev()
        .find(|(_, character)| {
            !character.is_ascii_alphanumeric() && *character != '_' && *character != '.'
        })
        .map_or(0, |(index, character)| index + character.len_utf8());
    let receiver = dot.get(start..)?.trim_matches('.');
    (!receiver.is_empty()).then(|| receiver.to_owned())
}

fn identifier_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || value == b'_'
}

fn statement_context(document: &NavigationDocument, offset: usize) -> bool {
    let mut blocks = Vec::<bool>::new();
    let mut previous = None;
    for token in document
        .tokens
        .iter()
        .take_while(|token| token.span.offset < offset)
    {
        previous = Some(&token.token);
        match token.token {
            Token::Begin | Token::Case | Token::Repeat => blocks.push(true),
            Token::Record | Token::Enum => blocks.push(false),
            Token::End => {
                blocks.pop();
            }
            Token::Until if blocks.last() == Some(&true) => {
                blocks.pop();
            }
            _ => {}
        }
    }
    blocks.last().copied().unwrap_or(false)
        || previous.is_some_and(|token| matches!(token, Token::Then | Token::Else | Token::Do))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceState {
    Code,
    String,
    LineComment,
}

fn is_code_offset(source: &str, offset: usize) -> bool {
    let bytes = source.as_bytes();
    let mut state = SourceState::Code;
    let mut index = 0;
    while index < offset {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        match state {
            SourceState::Code => match (current, next) {
                (b'\'', _) => state = SourceState::String,
                (b'/', Some(b'/')) => {
                    state = SourceState::LineComment;
                    index += 1;
                }
                _ => {}
            },
            SourceState::String => {
                if current == b'\'' {
                    if next == Some(b'\'') {
                        index += 1;
                    } else {
                        state = SourceState::Code;
                    }
                }
            }
            SourceState::LineComment => {
                if matches!(current, b'\n' | b'\r') {
                    state = SourceState::Code;
                }
            }
        }
        index += 1;
    }
    state == SourceState::Code
}
