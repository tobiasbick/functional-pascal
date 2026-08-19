//! Attaches every source comment to emission anchors (leading or same-line trailing).
//!
//! **Documentation:** [`docs/pascal/tools/fmt-style.md#comments`](../../../../docs/pascal/tools/fmt-style.md#comments)

use std::collections::{BTreeMap, BTreeSet};

use fpas_lexer::SourceComment;
use fpas_parser::CompilationUnit;

use super::anchors::{trailing_gap_allows, uses_keyword_offset};
use super::traversal;
use crate::FormatError;

/// Comments grouped by where they are emitted during formatting.
#[derive(Debug, Default, Clone)]
pub struct CommentMap {
    leading: BTreeMap<usize, Vec<LeadingComment>>,
    leading_blank_after: BTreeMap<usize, bool>,
    trailing: BTreeMap<usize, Vec<String>>,
    /// Leading comments with no following anchor (e.g. after `end.`).
    trailing_end: Vec<String>,
    uses_anchor: Option<usize>,
    body_anchors: BTreeMap<usize, usize>,
    header_anchors: BTreeMap<usize, usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct LeadingComment {
    pub(crate) text: String,
    pub(crate) blank_before: bool,
}

#[derive(Debug)]
struct PendingLeadingComment {
    start: usize,
    end: usize,
    text: String,
}

impl CommentMap {
    /// Builds a map from `source` and a parsed compilation unit.
    pub fn build(source: &str, unit: &CompilationUnit) -> Result<Self, FormatError> {
        let comments = fpas_lexer::collect_comments(source);
        let anchors = traversal::collect(unit, source);
        validate_anchors(source, &anchors)?;
        let mut map = Self::attach(
            source,
            &comments,
            &anchors.leading,
            &anchors.emission,
            &anchors.declarations,
        )?;
        map.uses_anchor = uses_keyword_offset(source);
        map.body_anchors = anchors.bodies;
        map.header_anchors = anchors.headers;
        Ok(map)
    }

    /// Returns leading comment texts for `anchor_offset`, in source order.
    #[must_use]
    pub(crate) fn leading_at(&self, anchor_offset: usize) -> &[LeadingComment] {
        self.leading.get(&anchor_offset).map_or(&[], Vec::as_slice)
    }

    /// Returns whether the source separated a leading comment group from its declaration.
    #[must_use]
    pub(crate) fn leading_needs_blank_after(&self, anchor_offset: usize) -> Option<bool> {
        self.leading_blank_after.get(&anchor_offset).copied()
    }

    /// Returns same-line trailing comment texts keyed by construct start offset.
    #[must_use]
    pub fn trailing_at(&self, anchor_start: usize) -> &[String] {
        self.trailing.get(&anchor_start).map_or(&[], Vec::as_slice)
    }

    /// Byte offset of the `uses` keyword when the unit had a uses clause in source.
    #[must_use]
    pub fn uses_anchor(&self) -> Option<usize> {
        self.uses_anchor
    }

    /// Byte offset of the `begin` keyword belonging to a program, routine, or closure owner.
    #[must_use]
    pub fn body_anchor(&self, owner_start: usize) -> Option<usize> {
        self.body_anchors.get(&owner_start).copied()
    }

    /// Trailing-comment anchor for a compilation-unit or routine header.
    #[must_use]
    pub fn header_anchor(&self, owner_start: usize) -> Option<usize> {
        self.header_anchors.get(&owner_start).copied()
    }

    /// Returns comments that trailed the compilation unit with no following anchor.
    #[must_use]
    pub fn trailing_end(&self) -> &[String] {
        &self.trailing_end
    }

    fn attach(
        source: &str,
        comments: &[SourceComment],
        leading_anchors: &[usize],
        emission_anchors: &[super::anchors::EmissionAnchor],
        declaration_anchors: &BTreeSet<usize>,
    ) -> Result<Self, FormatError> {
        let mut leading: BTreeMap<usize, Vec<PendingLeadingComment>> = BTreeMap::new();
        let mut trailing: BTreeMap<usize, Vec<(usize, String)>> = BTreeMap::new();
        let mut trailing_end: Vec<(usize, String)> = Vec::new();
        let mut previous_trailing: Option<(usize, usize)> = None;

        for comment in comments {
            let end_offset = comment
                .end_offset()
                .ok_or_else(|| invalid_comment_span(comment, source))?;
            let text = comment
                .text(source)
                .ok_or_else(|| invalid_comment_span(comment, source))?;
            let text = format_comment_text(text);
            let is_end_of_line = comment
                .is_end_of_line(source)
                .ok_or_else(|| invalid_comment_span(comment, source))?;
            if is_end_of_line {
                let direct = find_trailing_anchor(source, emission_anchors, comment.span.offset);
                let continued = previous_trailing.and_then(|(previous_end, anchor)| {
                    super::anchors::same_line(source, previous_end, comment.span.offset)
                        .then_some(anchor)
                });
                if let Some(start) = direct.or(continued) {
                    trailing
                        .entry(start)
                        .or_default()
                        .push((comment.span.offset, text));
                    previous_trailing = Some((end_offset, start));
                } else if let Some(anchor) = next_leading_anchor(leading_anchors, end_offset) {
                    leading
                        .entry(anchor)
                        .or_default()
                        .push(PendingLeadingComment {
                            start: comment.span.offset,
                            end: end_offset,
                            text,
                        });
                    previous_trailing = None;
                } else {
                    trailing_end.push((comment.span.offset, text));
                    previous_trailing = None;
                }
                continue;
            }

            previous_trailing = None;

            if let Some(anchor) = leading_anchors
                .iter()
                .copied()
                .filter(|anchor| *anchor > end_offset)
                .min()
            {
                leading
                    .entry(anchor)
                    .or_default()
                    .push(PendingLeadingComment {
                        start: comment.span.offset,
                        end: end_offset,
                        text,
                    });
            } else {
                trailing_end.push((comment.span.offset, text));
            }
        }

        let (leading, leading_blank_after) = prepare_leading(source, leading, declaration_anchors);
        Ok(Self {
            leading,
            leading_blank_after,
            trailing: sort_grouped(trailing),
            trailing_end: sort_entries(trailing_end),
            uses_anchor: None,
            body_anchors: BTreeMap::new(),
            header_anchors: BTreeMap::new(),
        })
    }
}

fn prepare_leading(
    source: &str,
    grouped: BTreeMap<usize, Vec<PendingLeadingComment>>,
    declaration_anchors: &BTreeSet<usize>,
) -> (BTreeMap<usize, Vec<LeadingComment>>, BTreeMap<usize, bool>) {
    let mut leading = BTreeMap::new();
    let mut blank_after = BTreeMap::new();
    for (anchor, mut entries) in grouped {
        entries.sort_by_key(|entry| entry.start);
        let mut previous_end = None;
        let prepared = entries
            .iter()
            .map(|entry| {
                let blank_before = previous_end.is_some_and(|end| {
                    logical_line_breaks(source.get(end..entry.start).unwrap_or_default()) > 1
                });
                previous_end = Some(entry.end);
                LeadingComment {
                    text: entry.text.clone(),
                    blank_before,
                }
            })
            .collect();
        if declaration_anchors.contains(&anchor) {
            let last_end = entries.last().map_or(anchor, |entry| entry.end);
            blank_after.insert(
                anchor,
                logical_line_breaks(source.get(last_end..anchor).unwrap_or_default()) != 1,
            );
        }
        leading.insert(anchor, prepared);
    }
    (leading, blank_after)
}

fn logical_line_breaks(text: &str) -> usize {
    let mut count = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                count += 1;
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\n' => count += 1,
            _ => {}
        }
    }
    count
}

fn next_leading_anchor(leading_anchors: &[usize], comment_end: usize) -> Option<usize> {
    leading_anchors
        .iter()
        .copied()
        .find(|anchor| *anchor > comment_end)
}

fn find_trailing_anchor(
    source: &str,
    anchors: &[super::anchors::EmissionAnchor],
    comment_start: usize,
) -> Option<usize> {
    anchors
        .iter()
        .filter(|anchor| trailing_gap_allows(source, anchor.end, comment_start))
        .max_by_key(|anchor| anchor.end)
        .map(|anchor| anchor.start)
}

fn sort_grouped(grouped: BTreeMap<usize, Vec<(usize, String)>>) -> BTreeMap<usize, Vec<String>> {
    grouped
        .into_iter()
        .map(|(anchor, entries)| (anchor, sort_entries(entries)))
        .collect()
}

fn sort_entries(mut entries: Vec<(usize, String)>) -> Vec<String> {
    entries.sort_by_key(|(offset, _)| *offset);
    entries.into_iter().map(|(_, text)| text).collect()
}

fn format_comment_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end()
        .to_string()
}

fn invalid_comment_span(comment: &SourceComment, source: &str) -> FormatError {
    FormatError::InvalidSourceSpan {
        offset: comment.span.offset,
        length: comment.span.length,
        source_len: source.len(),
    }
}

fn validate_anchors(
    source: &str,
    anchors: &traversal::CollectedAnchors,
) -> Result<(), FormatError> {
    for &offset in &anchors.leading {
        validate_span(source, offset, 0)?;
    }
    for anchor in &anchors.emission {
        validate_span(
            source,
            anchor.start,
            anchor.end.saturating_sub(anchor.start),
        )?;
    }
    for (&owner, &body) in &anchors.bodies {
        validate_span(source, owner, 0)?;
        validate_span(source, body, 0)?;
    }
    for (&owner, &header) in &anchors.headers {
        validate_span(source, owner, 0)?;
        validate_span(source, header, 0)?;
    }
    Ok(())
}

fn validate_span(source: &str, offset: usize, length: usize) -> Result<(), FormatError> {
    let end = offset.checked_add(length);
    if end.is_none_or(|end| source.get(offset..end).is_none()) {
        return Err(FormatError::InvalidSourceSpan {
            offset,
            length,
            source_len: source.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::CommentMap;
    use fpas_parser::parse_compilation_unit;

    #[test]
    fn attaches_line_comments_to_following_declarations() -> Result<(), String> {
        let source = "// Unit doc.\nunit Demo;\n\n// field doc\nmutable var Count: integer := 0;\n";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let map = CommentMap::build(source, &unit).map_err(|error| error.to_string())?;
        let unit_anchor = match &unit {
            fpas_parser::CompilationUnit::Unit(unit) => unit.span.offset,
            _ => return Err("expected unit".to_string()),
        };
        assert_eq!(map.leading_at(unit_anchor)[0].text, "// Unit doc.");
        let decl_anchor = crate::span::decl_span(match &unit {
            fpas_parser::CompilationUnit::Unit(unit) => &unit.declarations[0],
            _ => return Err("expected unit".to_string()),
        });
        assert_eq!(map.leading_at(decl_anchor)[0].text, "// field doc");
        Ok(())
    }

    #[test]
    fn preserves_comments_before_uses_begin_and_end_of_line() -> Result<(), String> {
        let source = "program T;\n// before uses\nuses Std.Console;\n\n// before begin\nbegin\n  WriteLn('ok') // trail\nend. // tail";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let map = CommentMap::build(source, &unit).map_err(|error| error.to_string())?;
        let Some(uses_anchor) = map.uses_anchor() else {
            return Err("expected uses anchor".to_string());
        };
        assert_eq!(map.leading_at(uses_anchor)[0].text, "// before uses");
        let program_anchor = match &unit {
            fpas_parser::CompilationUnit::Program(program) => program.span.offset,
            _ => return Err("expected program".to_string()),
        };
        let Some(begin_anchor) = map.body_anchor(program_anchor) else {
            return Err("expected begin anchor".to_string());
        };
        assert_eq!(map.leading_at(begin_anchor)[0].text, "// before begin");

        let formatted = crate::format_source(source, &unit).map_err(|error| error.to_string())?;
        assert!(formatted.contains("// before uses"));
        assert!(formatted.contains("// before begin"));
        assert!(formatted.contains("// trail"));
        Ok(())
    }

    #[test]
    fn attaches_leading_and_trailing_comments_to_uses_items() -> Result<(), String> {
        let source = "program T;\nuses Std.Console, // io\n// conversions\nStd.Conv;\nbegin\nend.";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let map = CommentMap::build(source, &unit).map_err(|error| error.to_string())?;
        let program = match &unit {
            fpas_parser::CompilationUnit::Program(program) => program,
            _ => return Err("expected program".to_string()),
        };

        assert_eq!(
            map.trailing_at(program.uses[0].span.offset),
            &["// io".to_string()]
        );
        assert_eq!(
            map.leading_at(program.uses[1].span.offset)[0].text,
            "// conversions"
        );
        Ok(())
    }

    #[test]
    fn preserves_line_comments_before_statements() {
        let source = "program T; begin\n  // setup\n  WriteLn('ok')\nend.";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let formatted = crate::format_source(source, &unit).expect("matching source and AST");
        assert!(formatted.contains("// setup"));
    }
}
