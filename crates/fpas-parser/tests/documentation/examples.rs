//! Checks AP13 examples in `docs/pascal/`, supplying only the missing context.

use super::markdown::pascal_blocks;
use fpas_parser::parse;

#[test]
fn conversion_math_and_path_examples_parse_with_statement_terminators() {
    for (path, markdown) in [
        (
            "docs/pascal/std/text/conv.md",
            include_str!("../../../../docs/pascal/std/text/conv.md"),
        ),
        (
            "docs/pascal/std/numeric/math.md",
            include_str!("../../../../docs/pascal/std/numeric/math.md"),
        ),
        (
            "docs/pascal/std/host/path.md",
            include_str!("../../../../docs/pascal/std/host/path.md"),
        ),
        (
            "docs/pascal/std/console/output.md",
            include_str!("../../../../docs/pascal/std/console/output.md"),
        ),
    ] {
        let blocks = pascal_blocks(markdown);
        assert!(!blocks.is_empty(), "no examples in {path}");
        for block in blocks {
            let source = if block.source.starts_with("program ") {
                block.source
            } else {
                format!("program Doc; begin\n{}end.", block.source)
            };
            let (_, errors) = parse(&source);
            assert!(errors.is_empty(), "{path}:{}\n{errors:#?}", block.line);
        }
    }
}

#[test]
fn console_record_examples_terminate_the_last_field() {
    let markdown = include_str!("../../../../docs/pascal/std/console/types.md");
    let blocks = pascal_blocks(markdown);
    let declarations = blocks
        .iter()
        .filter(|block| block.source.starts_with("type "))
        .collect::<Vec<_>>();
    assert_eq!(
        declarations.len(),
        2,
        "both conceptual records must be checked"
    );
    for block in declarations {
        let (_, errors) = parse(&format!("program Doc;\n{}begin end.", block.source));
        assert!(errors.is_empty(), "types.md:{}\n{errors:#?}", block.line);
    }
}

#[test]
fn screen_example_terminates_the_last_call_before_the_program_closer() {
    let markdown = include_str!("../../../../docs/pascal/std/console/screen-misc.md");
    let blocks = pascal_blocks(markdown);
    assert_eq!(blocks.len(), 1);
    let (_, errors) = parse(&format!("program Doc;\n{}", blocks[0].source));
    assert!(errors.is_empty(), "screen-misc.md\n{errors:#?}");
}

#[test]
fn bound_method_example_terminates_the_last_call() {
    let markdown = include_str!("../../../../docs/pascal/language/types/record-methods.md");
    let section = markdown
        .split("## Bound methods as values")
        .nth(1)
        .expect("bound-method section")
        .split("## Static routines")
        .next()
        .expect("bound-method section body");
    let blocks = pascal_blocks(section);
    assert_eq!(blocks.len(), 1);
    let (_, errors) = parse(&format!("program Doc;\n{}", blocks[0].source));
    assert!(errors.is_empty(), "record-methods.md\n{errors:#?}");
}

#[test]
fn tui_case_arm_examples_start_with_when() {
    let markdown = include_str!("../../../../docs/pascal/std/tui/application.md");
    let blocks = pascal_blocks(markdown);
    // Signature lists and constructor fragments are schematic, not complete case arms.
    let arms = blocks
        .iter()
        .filter(|block| block.source.lines().nth(1) == Some("begin"))
        .collect::<Vec<_>>();
    assert_eq!(
        arms.len(),
        3,
        "quit, started, and tick arms must be checked"
    );
    for block in arms {
        let source = format!(
            "program Doc; begin case Msg of\n{}end case; end.",
            block.source
        );
        let (_, errors) = parse(&source);
        assert!(
            errors.is_empty(),
            "application.md:{}\n{errors:#?}",
            block.line
        );
    }
}
