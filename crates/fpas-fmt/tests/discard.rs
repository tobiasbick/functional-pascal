//! Explicit discard retains operands, closure endings, comments, and terminators.

use fpas_parser::parse_compilation_unit;

#[test]
fn discard_round_trip_preserves_comments_and_closure_endings() {
    let source = "program T; begin\n// intentional\nDISCARD Make().GetValue(); // ignored result\ndiscard function(): integer begin return 1; end function;\nend.";
    let (unit, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit).unwrap();
    assert!(formatted.contains("discard Make().GetValue(); // ignored result"));
    assert!(formatted.contains("// intentional"));
    assert!(formatted.contains("discard function(): integer"));
    let (unit, errors) = parse_compilation_unit(&formatted);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        fpas_fmt::format_source(&formatted, &unit).unwrap(),
        formatted
    );
}
