//! Token-boundary layout hints, independent of indentation and display width.

use fpas_lexer::{Span, Token};
use std::collections::BTreeMap;

/// Matching delimiters of an expression or parameter list.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Group {
    pub(crate) open: usize,
    pub(crate) close: usize,
}

/// Original line breaks between tokens; newlines inside literals are excluded.
#[derive(Debug, Default, Clone)]
pub(crate) struct SourceLayout {
    tokens: Vec<Span>,
    breaks: Vec<usize>,
    groups: BTreeMap<usize, Group>,
    blank_before: BTreeMap<usize, bool>,
}

impl SourceLayout {
    /// Reads source layout without retaining author indentation.
    pub(crate) fn build(source: &str) -> Self {
        let (tokens, comments, _) = fpas_lexer::lex_with_comments(source);
        let mut layout = Self::default();
        let mut opens = Vec::new();
        for (index, token) in tokens.iter().enumerate() {
            layout.tokens.push(token.span);
            if index > 0 && tokens[index - 1].end.line < token.span.line {
                layout.breaks.push(token.span.offset);
            }
            match token.token {
                Token::LParen | Token::LBracket => opens.push(token.span.offset),
                Token::RParen | Token::RBracket => {
                    if let Some(open) = opens.pop() {
                        layout.groups.insert(
                            token.span.offset,
                            Group {
                                open,
                                close: token.span.offset,
                            },
                        );
                    }
                }
                _ => {}
            }
        }
        // Comment-internal newlines do not create blank statement sections.
        let mut items: Vec<_> = tokens
            .iter()
            .map(|t| (t.span.offset, t.span.line, t.end.line, false))
            .collect();
        items.extend(comments.iter().map(|c| {
            let text = c.text(source).unwrap_or_default();
            let lines = text.chars().filter(|ch| *ch == '\n').count()
                + text
                    .as_bytes()
                    .windows(2)
                    .filter(|pair| pair[0] == b'\r' && pair[1] != b'\n')
                    .count()
                + usize::from(text.ends_with('\r'));
            (
                c.span.offset,
                c.span.line,
                c.span
                    .line
                    .saturating_add(u32::try_from(lines).unwrap_or(u32::MAX)),
                true,
            )
        }));
        items.sort_unstable_by_key(|item| item.0);
        let mut pending_blank = false;
        for pair in items.windows(2) {
            pending_blank |= pair[1].1 > pair[0].2 + 1;
            layout.blank_before.insert(pair[1].0, pending_blank);
            if !pair[1].3 {
                pending_blank = false;
            }
        }
        layout
    }

    /// Returns whether a token begins on a later line than its preceding token.
    pub(crate) fn break_before(&self, offset: usize) -> bool {
        self.breaks.binary_search(&offset).is_ok()
    }

    /// Returns whether list boundaries have authored line breaks.
    pub(crate) fn list_breaks(
        &self,
        group: Option<Group>,
        offsets: impl Iterator<Item = usize>,
    ) -> bool {
        group.is_some_and(|g| self.break_before(g.close))
            || offsets.into_iter().any(|offset| self.break_before(offset))
    }

    /// Returns whether token boundaries in the given byte interval contain a line break.
    pub(crate) fn breaks_between(&self, start: usize, end: usize) -> bool {
        let index = self.breaks.partition_point(|offset| *offset <= start);
        self.breaks.get(index).is_some_and(|offset| *offset <= end)
    }

    /// Returns whether a statement or declaration starts a source section.
    pub(crate) fn blank_before(&self, offset: usize) -> bool {
        self.blank_before.get(&offset).copied().unwrap_or(false)
    }

    /// Finds the final delimiter pair contained by an expression or call statement.
    pub(crate) fn group(&self, span: Span) -> Option<Group> {
        let end = span.offset.saturating_add(span.length);
        self.groups
            .range(span.offset..end)
            .next_back()
            .map(|(_, group)| *group)
    }

    /// Finds the innermost delimiter pair containing all list items.
    pub(crate) fn enclosing(&self, first: Span, last: Span) -> Option<Group> {
        let end = last.offset.saturating_add(last.length);
        self.groups
            .range(end..)
            .find_map(|(_, group)| (group.open < first.offset).then_some(*group))
    }

    /// Returns the operator token between two binary operands.
    pub(crate) fn operator_between(&self, left_end: usize, right_start: usize) -> Option<Span> {
        let index = self.tokens.partition_point(|span| span.offset < left_end);
        self.tokens
            .get(index)
            .copied()
            .filter(|span| span.offset < right_start)
    }
}
