//! Checks the parseable correction pairs in `docs/pascal/tools/diagnostics.md`.

use super::markdown::catalog_example;
use fpas_parser::parse;

const CATALOG: &str = include_str!("../../../../docs/pascal/tools/diagnostics.md");

fn in_program(example: &str) -> String {
    if example.starts_with("program ") {
        example.to_owned()
    } else if example.starts_with("begin ") {
        format!("program Doc; {example}")
    } else if example.starts_with("type ") {
        format!("program Doc; {example} begin end.")
    } else {
        format!("program Doc; begin {example} end.")
    }
}

#[test]
fn positive_parser_corrections_use_current_terminators_and_closers() {
    for code in [
        "FP2001", "FP2002", "FP2003", "FP2004", "FP2005", "FP2006", "FP2012", "FP2013",
    ] {
        let source = in_program(catalog_example(CATALOG, code, true));
        let (_, errors) = parse(&source);
        assert!(errors.is_empty(), "{code}: {source}\n{errors:#?}");
    }
}

#[test]
fn wrong_parser_examples_still_emit_the_catalogued_diagnostic() {
    for code in [
        "FP2001", "FP2002", "FP2003", "FP2004", "FP2005", "FP2006", "FP2012", "FP2013",
    ] {
        let source = in_program(catalog_example(CATALOG, code, false));
        let (_, errors) = parse(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.as_diagnostic().code.to_string() == code),
            "{code}: {source}\n{errors:#?}"
        );
    }
}
