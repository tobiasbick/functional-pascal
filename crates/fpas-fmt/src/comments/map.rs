//! Attaches leading doc and block comments to declaration anchors.

use std::collections::BTreeMap;

use fpas_lexer::SourceComment;
use fpas_parser::{CompilationUnit, Decl, RecordMethod};

/// Comments to emit immediately before an anchor offset in source.
#[derive(Debug, Default, Clone)]
pub struct CommentMap {
    by_anchor: BTreeMap<usize, Vec<String>>,
}

impl CommentMap {
    /// Builds a map from `source` and a parsed compilation unit.
    #[must_use]
    pub fn build(source: &str, unit: &CompilationUnit) -> Self {
        let comments = fpas_lexer::collect_comments(source);
        let anchors = collect_anchors(unit);
        Self::attach(source, &comments, &anchors)
    }

    /// Returns comment texts attached to `anchor_offset`, in source order.
    #[must_use]
    pub fn comments_at(&self, anchor_offset: usize) -> &[String] {
        self.by_anchor
            .get(&anchor_offset)
            .map_or(&[], Vec::as_slice)
    }

    fn attach(source: &str, comments: &[SourceComment], anchors: &[usize]) -> Self {
        let mut grouped: BTreeMap<usize, Vec<(usize, String)>> = BTreeMap::new();

        for comment in comments {
            if !comment.is_preservable() || comment.is_end_of_line(source) {
                continue;
            }
            let Some(anchor) = nearest_following_anchor(comment.end_offset(), anchors, source)
            else {
                continue;
            };
            grouped.entry(anchor).or_default().push((
                comment.span.offset,
                normalize_comment_text(comment.text(source)),
            ));
        }

        let by_anchor = grouped
            .into_iter()
            .map(|(anchor, mut entries)| {
                entries.sort_by_key(|(offset, _)| *offset);
                (anchor, entries.into_iter().map(|(_, text)| text).collect())
            })
            .collect();

        Self { by_anchor }
    }
}

fn nearest_following_anchor(comment_end: usize, anchors: &[usize], source: &str) -> Option<usize> {
    anchors
        .iter()
        .copied()
        .filter(|anchor| *anchor > comment_end && gap_leads_to_anchor(source, comment_end, *anchor))
        .min()
}

/// True when `source[comment_end..anchor]` is only whitespace or declaration keywords
/// (`private`, `public`, `mutable var`, `var`, `const`, `type`, …) before the parser anchor.
fn gap_leads_to_anchor(source: &str, comment_end: usize, anchor: usize) -> bool {
    let gap = source.get(comment_end..anchor).unwrap_or("");
    if gap.chars().all(char::is_whitespace) {
        return true;
    }
    is_declaration_preamble(gap.trim())
}

fn is_declaration_preamble(gap: &str) -> bool {
    let mut rest = gap;
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return true;
        }
        if let Some(next) = rest.strip_prefix("public") {
            rest = next;
            continue;
        }
        if let Some(next) = rest.strip_prefix("private") {
            rest = next;
            continue;
        }
        if let Some(next) = rest.strip_prefix("mutable") {
            rest = next.trim_start();
            if !rest.starts_with("var") {
                return false;
            }
            rest = rest.strip_prefix("var").unwrap_or("");
            return rest.trim().is_empty();
        }
        for keyword in ["var", "const", "type", "function", "procedure"] {
            if let Some(next) = rest.strip_prefix(keyword) {
                return next.trim().is_empty();
            }
        }
        return false;
    }
}

fn normalize_comment_text(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn collect_anchors(unit: &CompilationUnit) -> Vec<usize> {
    match unit {
        CompilationUnit::Program(program) => {
            collect_decl_anchors(program.span.offset, &program.declarations)
        }
        CompilationUnit::Unit(unit) => collect_decl_anchors(unit.span.offset, &unit.declarations),
    }
}

fn collect_decl_anchors(root_offset: usize, decls: &[Decl]) -> Vec<usize> {
    let mut anchors = vec![root_offset];
    for decl in decls {
        push_decl_anchors(&mut anchors, decl);
    }
    anchors.sort_unstable();
    anchors.dedup();
    anchors
}

fn push_decl_anchors(out: &mut Vec<usize>, decl: &Decl) {
    out.push(crate::span::decl_span(decl));
    if let Decl::TypeDef(type_def) = decl
        && let fpas_parser::TypeBody::Record(record) = &type_def.body
    {
        for method in &record.methods {
            match method {
                RecordMethod::Function(function) => out.push(function.span.offset),
                RecordMethod::Procedure(procedure) => out.push(procedure.span.offset),
            }
        }
    }
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
        assert_eq!(map.comments_at(unit_anchor), ["/// Unit doc."]);
        let decl_anchor = crate::span::decl_span(match &unit {
            fpas_parser::CompilationUnit::Unit(unit) => &unit.declarations[0],
            _ => panic!("expected unit"),
        });
        assert_eq!(map.comments_at(decl_anchor), ["{ field doc }"]);
    }
}
