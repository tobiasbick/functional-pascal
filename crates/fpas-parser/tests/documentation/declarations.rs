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

fn optional_handler_example() -> String {
    let markdown = include_str!("../../../../docs/pascal/language/functions/first-class.md");
    let section = markdown
        .split("## Optional handlers")
        .nth(1)
        .expect("optional handler section");
    let blocks = pascal_blocks(section);
    assert_eq!(blocks.len(), 1, "optional handler example fence");
    blocks[0].source.clone()
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
fn optional_handler_types_each_have_their_own_type_keyword() {
    let source = optional_handler_example();
    let (program, errors) = parse(&source);
    assert!(errors.is_empty(), "first-class.md\n{source}\n{errors:#?}");
    assert_eq!(
        program.declarations.len(),
        3,
        "ClickHandler, Button, and HandleClick declarations"
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
fn grouped_optional_handler_record_reports_fp2015() {
    let source = optional_handler_example();
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
