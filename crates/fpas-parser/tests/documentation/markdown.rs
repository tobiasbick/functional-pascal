//! Extracts handbook literals without changing their syntax.

/// One Pascal code fence and its first source line in the handbook.
pub(super) struct CodeBlock {
    /// One-based Markdown line of the first source statement.
    pub(super) line: usize,
    /// Verbatim fenced source with normalized line endings.
    pub(super) source: String,
}

/// Extracts Pascal/fpas fences, preserving statements and trailing comments.
pub(super) fn pascal_blocks(markdown: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut fence: Option<(usize, bool, String)> = None;
    for (index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if let Some((start, is_pascal, source)) = &mut fence {
            if trimmed == "```" {
                if *is_pascal {
                    blocks.push(CodeBlock {
                        line: *start,
                        source: std::mem::take(source),
                    });
                }
                fence = None;
            } else {
                source.push_str(line);
                source.push('\n');
            }
        } else if let Some(language) = trimmed.strip_prefix("```") {
            fence = Some((
                index + 2,
                matches!(language, "pascal" | "fpas"),
                String::new(),
            ));
        }
    }
    assert!(fence.is_none(), "unterminated Markdown code fence");
    blocks
}

/// Reads the first inline code example from one diagnostic catalog column.
pub(super) fn catalog_example<'a>(markdown: &'a str, code: &str, corrected: bool) -> &'a str {
    let prefix = format!("| {code} |");
    let rows = markdown
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "missing or duplicate catalog row: {code}");
    let column = if corrected { 4 } else { 3 };
    let cell = rows[0].split('|').nth(column).expect("example column");
    cell.split('`').nth(1).expect("inline code example")
}

#[test]
fn extraction_keeps_comments_and_skips_non_pascal_fences() {
    let blocks = pascal_blocks(
        "```text\n```pascal\n```\n\n```pascal\nWriteLn(); // comment\n```\n```fpas\nnull;\n```\n",
    );
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].line, 6);
    assert_eq!(blocks[0].source, "WriteLn(); // comment\n");
    assert_eq!(blocks[1].source, "null;\n");
}
