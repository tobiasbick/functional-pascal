//! Reads complete fenced examples for CLI handbook tests.

/// One fenced example with its Markdown location.
pub(super) struct CodeBlock {
    /// One-based line of the first source statement.
    pub(super) line: usize,
    /// First fence-info word; callers validate the accepted language.
    pub(super) language: String,
    /// Source with normalized line endings, otherwise unchanged.
    pub(super) source: String,
}

/// Extracts every backtick fence so a missing or misspelled language cannot hide an example.
pub(super) fn code_blocks(markdown: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<CodeBlock> = None;
    for (index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(block) = &mut current {
            if trimmed == "```" {
                blocks.push(current.take().expect("open fence"));
            } else {
                block.source.push_str(line);
                block.source.push('\n');
            }
        } else if let Some(language) = trimmed.strip_prefix("```") {
            current = Some(CodeBlock {
                line: index + 2,
                language: language.split_whitespace().next().unwrap_or("").to_owned(),
                source: String::new(),
            });
        } else {
            assert!(
                !trimmed.starts_with("~~~"),
                "use backtick fences at line {}",
                index + 1
            );
        }
    }
    assert!(current.is_none(), "unterminated Markdown code fence");
    blocks
}
