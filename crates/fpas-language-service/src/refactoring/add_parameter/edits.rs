//! Token-delimited insertions and exact coordinate mapping for semantic comparison.

use super::fragments::{Parameter, trailing_newline};
use crate::{CallRelation, DocumentSnapshot, DocumentSymbol, SourceEdit, SourceQueryError};
use fpas_diagnostics::SourceSpan;
use fpas_lexer::{SpannedToken, Token};
use std::{
    collections::BTreeMap,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};

pub(super) struct Insertion {
    pub edit: SourceEdit,
    /// Argument bytes relative to the inserted text, excluding separator, label and var marker.
    pub argument: Option<Range<usize>>,
}

pub(super) struct Insertions {
    pub items: Vec<Insertion>,
    coordinates: BTreeMap<PathBuf, Vec<Coordinate>>,
}

struct Coordinate {
    index: usize,
    start: usize,
    end: usize,
    delta: usize,
}

impl Insertions {
    pub(super) fn generate(
        declaration: &DocumentSymbol,
        snapshot: &Arc<DocumentSnapshot>,
        calls: &[CallRelation],
        parameter: &Parameter<'_>,
    ) -> Result<Self, SourceQueryError> {
        let mut tokens = BTreeMap::<PathBuf, Vec<SpannedToken>>::new();
        tokens.insert(snapshot.path().into(), fpas_lexer::lex(snapshot.source()).0);
        let declaration_tokens = &tokens[snapshot.path()];
        let opening = declaration_tokens
            .iter()
            .position(|token| token.span.offset >= declaration.selection_span.end())
            .filter(|index| matches!(declaration_tokens[*index].token, Token::LParen))
            .ok_or_else(|| malformed("The selected routine has no checked parameter delimiter."))?;
        let mut depth = 0usize;
        let closing = declaration_tokens[opening..]
            .iter()
            .find(|token| {
                match token.token {
                    Token::LParen => depth += 1,
                    Token::RParen => depth = depth.saturating_sub(1),
                    _ => {}
                }
                matches!(token.token, Token::RParen) && depth == 0
            })
            .ok_or_else(|| malformed("The checked routine's parameter list is unterminated."))?;
        let has_parameters = declaration
            .callable
            .as_ref()
            .is_some_and(|s| !s.parameters.is_empty());
        let declaration_text = format!(
            "{}{}{}: {}{}",
            if has_parameters { "; " } else { "" },
            parameter.marker(),
            parameter.name,
            parameter.type_text,
            trailing_newline(parameter.type_text, snapshot.source())
        );
        let mut items = vec![Insertion {
            edit: insert(snapshot, closing.span.offset, declaration_text),
            argument: None,
        }];
        let mut bytes = items[0].edit.new_text.len();
        for call in calls {
            let path = call.snapshot.path();
            let tokens = tokens
                .entry(path.into())
                .or_insert_with(|| fpas_lexer::lex(call.snapshot.source()).0);
            let end = tokens.partition_point(|token| token.span.offset < call.span.end());
            let closing = end
                .checked_sub(1)
                .and_then(|index| tokens.get(index))
                .filter(|token| {
                    matches!(token.token, Token::RParen) && token.span.offset >= call.span.offset()
                })
                .ok_or_else(|| {
                    malformed("A checked ordinary call has no closing argument delimiter.")
                })?;
            let mut text = if call.arguments.is_empty() {
                String::new()
            } else {
                ", ".into()
            };
            if call.arguments.first().is_some_and(|a| a.name.is_some()) {
                text.push_str(parameter.name);
                text.push_str(" := ");
            }
            text.push_str(parameter.marker());
            let start = text.len();
            text.push_str(parameter.argument);
            let end = text.len();
            text.push_str(trailing_newline(parameter.argument, call.snapshot.source()));
            bytes = bytes.saturating_add(text.len());
            if bytes > 128 * 1024 * 1024 {
                return Err(SourceQueryError::new(
                    "plan_limit",
                    "Parameter insertions exceed the preparation byte limit.",
                ));
            }
            items.push(Insertion {
                edit: insert(&call.snapshot, closing.span.offset, text),
                argument: Some(start..end),
            });
        }
        items.sort_by(|a, b| {
            a.edit
                .path
                .cmp(&b.edit.path)
                .then(a.edit.range.offset().cmp(&b.edit.range.offset()))
        });
        if items.windows(2).any(|pair| {
            pair[0].edit.path == pair[1].edit.path
                && pair[0].edit.range.offset() == pair[1].edit.range.offset()
        }) {
            return Err(malformed(
                "Several insertions select the same physical delimiter.",
            ));
        }
        let mut coordinates = BTreeMap::<PathBuf, Vec<Coordinate>>::new();
        for (index, item) in items.iter().enumerate() {
            let entries = coordinates.entry(item.edit.path.clone()).or_default();
            let delta = entries.last().map_or(0, |entry| entry.delta);
            let start = item.edit.range.offset() + delta;
            entries.push(Coordinate {
                index,
                start,
                end: start + item.edit.new_text.len(),
                delta: delta + item.edit.new_text.len(),
            });
        }
        Ok(Self { items, coordinates })
    }

    pub(super) fn edits(&self) -> Vec<SourceEdit> {
        self.items.iter().map(|item| item.edit.clone()).collect()
    }

    /// New tokens have no original coordinate; other tokens map back exactly.
    pub(super) fn original(&self, path: &Path, offset: usize) -> Option<usize> {
        match self.coordinate(path, offset) {
            Some(entry) if offset < entry.end => None,
            Some(entry) => offset.checked_sub(entry.delta),
            None => Some(offset),
        }
    }

    pub(super) fn inserted(&self, path: &Path, offset: usize) -> Option<(usize, usize)> {
        let entry = self
            .coordinate(path, offset)
            .filter(|entry| offset < entry.end)?;
        Some((entry.index, offset - entry.start))
    }

    fn coordinate(&self, path: &Path, offset: usize) -> Option<&Coordinate> {
        let entries = self.coordinates.get(path)?;
        let end = entries.partition_point(|entry| entry.start <= offset);
        end.checked_sub(1).map(|index| &entries[index])
    }
}

fn insert(snapshot: &Arc<DocumentSnapshot>, offset: usize, new_text: String) -> SourceEdit {
    SourceEdit {
        path: snapshot.path().into(),
        snapshot: Arc::clone(snapshot),
        range: SourceSpan::new(offset, 0, 1, 1),
        new_text,
    }
}
fn malformed(message: &str) -> SourceQueryError {
    SourceQueryError::new("unsupported_call_syntax", message)
}
