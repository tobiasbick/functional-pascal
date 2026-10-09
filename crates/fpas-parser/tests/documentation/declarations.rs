//! AP11 declaration keywords in `docs/pascal/` examples and their original regressions.

use super::markdown::pascal_blocks;
use fpas_diagnostics::codes::PARSE_MISSING_DECLARATION_KEYWORD;
use fpas_parser::parse;

/// Wraps the complete other-types handbook snippet in a program.
pub(super) fn formatter_type_example() -> String {
    let markdown = include_str!("../../../../docs/pascal/tools/fmt-style.md");
    let section = markdown
        .split("## More examples — other types (snippet)")
        .nth(1)
        .expect("other-types section")
        .split("## Types (summary)")
        .next()
        .expect("other-types section body");
    let blocks = pascal_blocks(section);
    assert_eq!(blocks.len(), 1, "other-types example fence");
    format!("program Doc;\n{}\nbegin end.", blocks[0].source)
}

fn record_event_example() -> String {
    let markdown = include_str!("../../../../docs/pascal/language/types/record-events.md");
    let section = markdown
        .split("## Declaration")
        .nth(1)
        .expect("event declaration section")
        .split("## Assignment")
        .next()
        .expect("event declaration section body");
    let blocks = pascal_blocks(section);
    assert_eq!(blocks.len(), 1, "event declaration example fence");
    // Supply bodies for the schematic accessors without altering declaration prefixes.
    let declarations = blocks[0]
        .source
        .replace(
            "function ReadOnClick(Self: Button): Option of ClickHandler;",
            "function ReadOnClick(Self: Button): Option of ClickHandler;\n\
             begin return None; end function;",
        )
        .replace(
            "procedure WriteOnClick(Self: Button; Handler: Option of ClickHandler);",
            "procedure WriteOnClick(Self: Button; Handler: Option of ClickHandler);\n\
             begin null; end procedure;",
        );
    format!("program Doc;\n{declarations}begin end.")
}

#[test]
fn formatter_enums_each_have_their_own_type_keyword() {
    let source = formatter_type_example();
    let (program, errors) = parse(&source);
    assert!(errors.is_empty(), "fmt-style.md\n{source}\n{errors:#?}");
    assert_eq!(
        program.declarations.len(),
        3,
        "Color, Shape, and IntOption declarations"
    );
}

#[test]
fn record_event_types_each_have_their_own_type_keyword() {
    let source = record_event_example();
    let (program, errors) = parse(&source);
    assert!(errors.is_empty(), "record-events.md\n{source}\n{errors:#?}");
    assert_eq!(
        program.declarations.len(),
        2,
        "ClickHandler and Button declarations"
    );
}

#[test]
fn original_grouped_formatter_enum_reports_fp2015() {
    let source = formatter_type_example();
    assert!(source.contains("type Shape"), "explicit Shape declaration");
    let source = source.replacen("type Shape", "Shape", 1);
    let (_, errors) = parse(&source);
    assert_eq!(errors.len(), 1, "{source}\n{errors:#?}");
    assert_eq!(
        errors[0].as_diagnostic().code,
        PARSE_MISSING_DECLARATION_KEYWORD
    );
}

#[test]
fn original_grouped_event_record_reports_fp2015() {
    let source = record_event_example();
    assert!(
        source.contains("type Button"),
        "explicit Button declaration"
    );
    let source = source.replacen("type Button", "Button", 1);
    let (_, errors) = parse(&source);
    assert_eq!(errors.len(), 1, "{source}\n{errors:#?}");
    assert_eq!(
        errors[0].as_diagnostic().code,
        PARSE_MISSING_DECLARATION_KEYWORD
    );
}
