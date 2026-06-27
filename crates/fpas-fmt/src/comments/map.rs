//! Attaches every source comment to emission anchors (leading or same-line trailing).
//!
//! **Documentation:** [`docs/pascal/tools/fmt-style.md#comments`](../../../../docs/pascal/tools/fmt-style.md#comments)

use std::collections::BTreeMap;

use fpas_lexer::SourceComment;
use fpas_parser::CompilationUnit;

use super::anchors::{
    begin_keyword_offset, collect_emission_anchors, collect_leading_anchor_offsets,
    trailing_gap_allows, uses_keyword_offset,
};

/// Comments grouped by where they are emitted during formatting.
#[derive(Debug, Default, Clone)]
pub struct CommentMap {
    leading: BTreeMap<usize, Vec<String>>,
    trailing: BTreeMap<usize, Vec<String>>,
    /// Leading comments with no following anchor (e.g. after `end.`).
    trailing_end: Vec<String>,
    uses_anchor: Option<usize>,
    begin_anchor: Option<usize>,
}

impl CommentMap {
    /// Builds a map from `source` and a parsed compilation unit.
    #[must_use]
    pub fn build(source: &str, unit: &CompilationUnit) -> Self {
        let comments = fpas_lexer::collect_comments(source);
        let leading_anchors = collect_leading_anchor_offsets(unit, source);
        let emission_anchors = collect_emission_anchors(unit);
        let mut map = Self::attach(source, &comments, &leading_anchors, &emission_anchors);
        map.uses_anchor = uses_keyword_offset(source);
        map.begin_anchor = begin_keyword_offset(source);
        map
    }

    /// Returns leading comment texts for `anchor_offset`, in source order.
    #[must_use]
    pub fn leading_at(&self, anchor_offset: usize) -> &[String] {
        self.leading.get(&anchor_offset).map_or(&[], Vec::as_slice)
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

    /// Byte offset of the program `begin` keyword when present in source.
    #[must_use]
    pub fn begin_anchor(&self) -> Option<usize> {
        self.begin_anchor
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
    ) -> Self {
        let mut leading: BTreeMap<usize, Vec<(usize, String)>> = BTreeMap::new();
        let mut trailing: BTreeMap<usize, Vec<(usize, String)>> = BTreeMap::new();
        let mut trailing_end: Vec<(usize, String)> = Vec::new();

        for comment in comments {
            let text = format_comment_text(comment.text(source));
            if comment.is_end_of_line(source) {
                if let Some(start) =
                    find_trailing_anchor(source, emission_anchors, comment.span.offset)
                {
                    trailing
                        .entry(start)
                        .or_default()
                        .push((comment.span.offset, text));
                } else {
                    trailing_end.push((comment.span.offset, text));
                }
                continue;
            }

            if let Some(anchor) = leading_anchors
                .iter()
                .copied()
                .filter(|anchor| *anchor > comment.end_offset())
                .min()
            {
                leading
                    .entry(anchor)
                    .or_default()
                    .push((comment.span.offset, text));
            } else {
                trailing_end.push((comment.span.offset, text));
            }
        }

        Self {
            leading: sort_grouped(leading),
            trailing: sort_grouped(trailing),
            trailing_end: sort_entries(trailing_end),
            uses_anchor: None,
            begin_anchor: None,
        }
    }
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
    text.replace("\r\n", "\n").trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::CommentMap;
    use fpas_parser::parse_compilation_unit;

    #[test]
    fn attaches_doc_and_block_comments_to_following_decl() {
        let source = "/// Unit doc.\nunit Demo;\n\n{ field doc }\nprivate mutable var Count: integer := 0;\n";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let map = CommentMap::build(source, &unit);
        let unit_anchor = match &unit {
            fpas_parser::CompilationUnit::Unit(unit) => unit.span.offset,
            _ => panic!("expected unit"),
        };
        assert_eq!(map.leading_at(unit_anchor), ["/// Unit doc."]);
        let decl_anchor = crate::span::decl_span(match &unit {
            fpas_parser::CompilationUnit::Unit(unit) => &unit.declarations[0],
            _ => panic!("expected unit"),
        });
        assert_eq!(map.leading_at(decl_anchor), ["{ field doc }"]);
    }

    #[test]
    fn preserves_comments_before_uses_begin_and_end_of_line() {
        let source = "program T;\n{ before uses }\nuses Std.Console;\n\n{ before begin }\nbegin\n  WriteLn('ok') // trail\nend. // tail";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let map = CommentMap::build(source, &unit);
        let uses_anchor = map.uses_anchor().expect("uses");
        assert_eq!(map.leading_at(uses_anchor), ["{ before uses }"]);
        let begin_anchor = map.begin_anchor().expect("begin");
        assert_eq!(map.leading_at(begin_anchor), ["{ before begin }"]);

        let formatted = crate::format_source(source, &unit);
        assert!(formatted.contains("{ before uses }"));
        assert!(formatted.contains("{ before begin }"));
        assert!(formatted.contains("// trail"));
    }

    #[test]
    fn preserves_line_comments_before_statements() {
        let source = "program T; begin\n  // setup\n  WriteLn('ok')\nend.";
        let (unit, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        let formatted = crate::format_source(source, &unit);
        assert!(formatted.contains("// setup"));
    }
}
