//! Markdown documentation attached to Functional Pascal declarations.
//!
//! **Documentation:** `docs/pascal/language/basics/comments.md`

use std::path::Path;

use fpas_lexer::SourceComment;

use crate::{LanguageService, LanguageServiceError};

impl LanguageService {
    /// Resolves Markdown documentation for a completion declaration identity.
    pub fn completion_documentation(
        &mut self,
        path: &Path,
        declaration_offset: usize,
        qualified_name: &str,
    ) -> Result<Option<String>, LanguageServiceError> {
        let known = self
            .workspace_symbol_index()?
            .all_locations()
            .into_iter()
            .any(|location| {
                location.path == path
                    && location.symbol.full_span.offset() == declaration_offset
                    && location
                        .symbol
                        .qualified_name
                        .eq_ignore_ascii_case(qualified_name)
            });
        if !known {
            return Ok(None);
        }
        let snapshot = self.snapshot(path)?;
        Ok(preceding_documentation(
            snapshot.source(),
            declaration_offset,
        ))
    }
}

/// Extracts an immediately preceding standalone `//` block as Markdown.
pub(crate) fn preceding_documentation(source: &str, declaration_offset: usize) -> Option<String> {
    let declaration_offset = declaration_offset.min(source.len());
    let declaration_line_start = source
        .get(..declaration_offset)?
        .rfind(['\n', '\r'])
        .map_or(0, |index| index + 1);
    let declaration_anchor = source
        .get(declaration_line_start..declaration_offset)?
        .find(|ch: char| !ch.is_whitespace())
        .map_or(declaration_offset, |index| declaration_line_start + index);
    let comments = fpas_lexer::collect_comments(source);
    let last_index = comments.iter().rposition(|comment| {
        comment
            .end_offset()
            .is_some_and(|end| end <= declaration_anchor)
    })?;
    let last = &comments[last_index];
    if last.is_end_of_line(source)?
        || !is_adjacent_line(source, last.end_offset()?, declaration_anchor)
    {
        return None;
    }

    let mut first_index = last_index;
    while first_index > 0 {
        let previous = &comments[first_index - 1];
        let current = &comments[first_index];
        if previous.is_end_of_line(source)?
            || !is_adjacent_line(source, previous.end_offset()?, current.span.offset)
        {
            break;
        }
        first_index -= 1;
    }

    comments[first_index..=last_index]
        .iter()
        .map(|comment| documentation_line(comment, source))
        .collect::<Option<Vec<_>>>()
        .map(|lines| lines.join("\n"))
}

fn documentation_line(comment: &SourceComment, source: &str) -> Option<String> {
    let text = comment.text(source)?.strip_prefix("//")?;
    Some(text.strip_prefix(' ').unwrap_or(text).to_owned())
}

fn is_adjacent_line(source: &str, left: usize, right: usize) -> bool {
    let Some(gap) = source.get(left..right) else {
        return false;
    };
    let mut line_breaks = 0;
    let mut chars = gap.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                line_breaks += 1;
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\n' => line_breaks += 1,
            _ if ch.is_whitespace() => {}
            _ => return false,
        }
    }
    line_breaks == 1
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::preceding_documentation;

    #[test]
    fn extracts_markdown_from_the_single_comment_syntax() {
        let source = "// Summary with `code`.\r\n//\r\n// - first\r\n  public type Item = integer;";
        let declaration = source.find("public").expect("declaration");
        assert_eq!(
            preceding_documentation(source, declaration).as_deref(),
            Some("Summary with `code`.\n\n- first")
        );
    }

    #[test]
    fn supports_lf_crlf_bare_cr_and_eof_comments() {
        for newline in ["\n", "\r\n", "\r"] {
            let source = format!("// docs{newline}type Item = integer;");
            let declaration = source.find("type").expect("declaration");
            assert_eq!(
                preceding_documentation(&source, declaration).as_deref(),
                Some("docs"),
                "{newline:?}"
            );
        }
        assert_eq!(preceding_documentation("// docs", 7), None);
    }

    #[test]
    fn blank_line_or_trailing_comment_does_not_attach() {
        for source in [
            "// detached\n\ntype Item = integer;",
            "var Value: integer := 1; // trailing\ntype Item = integer;",
        ] {
            let declaration = source.find("type").expect("declaration");
            assert_eq!(preceding_documentation(source, declaration), None);
        }
    }
}
