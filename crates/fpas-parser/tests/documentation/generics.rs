//! Generic type spellings documented in `docs/pascal/tools/fmt-style.md`.

use super::declarations::formatter_type_example;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
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
fn original_user_defined_generic_type_applications_report_fp2001() {
    let example = formatter_type_example();
    assert!(
        example.contains("type IntOption = option of integer;"),
        "documented built-in alias"
    );
    for application in ["Box of integer", "Box of string", "Pair of integer, string"] {
        let source = example.replacen("option of integer", application, 1);
        let (_, errors) = parse(&source);
        assert_eq!(errors.len(), 1, "{source}\n{errors:#?}");
        let diagnostic = errors[0].as_diagnostic();
        assert_eq!(diagnostic.code, PARSE_EXPECTED_TOKEN, "{source}");
        assert!(
            diagnostic
                .message
                .contains("User-defined generic type arguments"),
            "{source}\n{diagnostic:#?}"
        );
    }
}
