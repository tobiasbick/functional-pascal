//! Pascal source extraction from the repository's Markdown code fences.

/// Returns actual Pascal fences, leaving examples inside other fences alone.
pub(super) fn pascal_sources(markdown: &str) -> impl Iterator<Item = &str> {
    let mut remaining = markdown;
    std::iter::from_fn(move || {
        loop {
            let mut lines = remaining.split_inclusive('\n');
            let mut offset = 0;
            let (language, start) = loop {
                let line = lines.next()?;
                offset += line.len();
                if let Some(language) = line.trim().strip_prefix("```") {
                    break (language, offset);
                }
            };
            let source = &remaining[start..];
            let mut end = 0;
            let mut after = None;
            for line in source.split_inclusive('\n') {
                if line.trim() == "```" {
                    after = Some(end + line.len());
                    break;
                }
                end += line.len();
            }
            let after = after.expect("Markdown code fence must be closed");
            remaining = &source[after..];
            if language == "pascal" {
                return Some(&source[..end]);
            }
        }
    })
}

#[test]
fn reads_only_pascal_fences_and_preserves_source_bytes() {
    let markdown = "Text\r\n```text\r\n```pascal\r\n```\r\n\
        ```pascal\r\nprogram P;\r\nbegin null; end program;\r\n```\r\n\
        ```json\n{}\n```\n```pascal\nunit U; end unit;\n```";
    assert_eq!(
        pascal_sources(markdown).collect::<Vec<_>>(),
        [
            "program P;\r\nbegin null; end program;\r\n",
            "unit U; end unit;\n"
        ]
    );
}
