use crate::{ParseDiagnostic, parse};
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn type_applications_require_parentheses_and_dictionary_comma() {
    for ty in [
        "array of integer",
        "channel of string",
        "task of integer",
        "Option of integer",
        "Result of integer, string",
        "dict of string to integer",
        "dict of (string to integer)",
        "Box of integer",
    ] {
        let source = format!("program Types; type Item = {ty}; begin null; end program;");
        let (_, errors) = parse(&source);
        assert!(errors.iter().any(|error| matches!(error, ParseDiagnostic::Parser(error) if error.code == PARSE_EXPECTED_TOKEN)), "accepted `{ty}`: {errors:?}");
        if !ty.contains('(') {
            assert!(errors.iter().any(|error| matches!(error, ParseDiagnostic::Parser(error) if error.help.as_deref().is_some_and(|help| help.contains("array of (integer)")))), "missing concrete hint for `{ty}`: {errors:?}");
        }
    }
}

#[test]
fn generic_angle_headings_are_rejected_with_canonical_examples() {
    for declaration in [
        "type Box<T> = record Value: T; end record;",
        "function Identity<T>(Value: T): T; begin return Value; end function;",
        "procedure Ignore<T>(Value: T); begin null; end procedure;",
    ] {
        let source = format!("program Headings; {declaration} begin null; end program;");
        let (_, errors) = parse(&source);
        assert!(errors.iter().any(|error| matches!(error, ParseDiagnostic::Parser(error) if error.message == "Generic parameters use `of (...)`" && error.help.as_deref().is_some_and(|help| help.contains("Identity of (T)")))), "{errors:?}");
    }
}
