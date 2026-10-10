//! Generic type spellings documented in `docs/pascal/tools/fmt-style.md`.

use super::markdown::pascal_blocks;
use fpas_parser::parse;

#[test]
fn documented_builtin_generic_type_forms_parse() {
    let markdown = include_str!("../../../../docs/pascal/tools/fmt-style.md");
    let summary = markdown
        .lines()
        .find(|line| line.starts_with("- Built-in generic types:"))
        .expect("built-in type summary");
    let forms = summary.split('`').skip(1).step_by(2).collect::<Vec<_>>();
    assert_eq!(forms.len(), 6, "all documented built-in generic forms");
    for form in forms {
        let source = format!("program Doc;\ntype Example = {form};\nbegin end.");
        let (program, errors) = parse(&source);
        assert!(errors.is_empty(), "{source}\n{errors:#?}");
        assert_eq!(program.declarations.len(), 1, "{source}");
    }
}

#[test]
fn documented_generic_record_and_routine_examples_parse() {
    let markdown = include_str!("../../../../docs/pascal/language/types/generics.md");
    let blocks = pascal_blocks(markdown);
    assert!(!blocks.is_empty(), "generic examples");
    for block in blocks {
        let source = format!("program Doc;\n{}\nbegin end.", block.source);
        let (_, errors) = parse(&source);
        assert!(
            errors.is_empty(),
            "generics.md:{}\n{source}\n{errors:#?}",
            block.line
        );
    }
}
