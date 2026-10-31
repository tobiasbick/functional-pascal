//! Resource declarations retain their modifiers and comments through formatting.

#[test]
fn resource_declarations_round_trip() -> Result<(), fpas_fmt::FormatError> {
    let source = "program T; type R = record\n// cleanup\nrelease procedure Close(Self: R); begin null; end procedure;\nend record; begin\n// acquire\nusing const Value := R(); // owned\nend.";
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit)?;
    assert!(
        formatted.contains("release procedure Close(Self: R)"),
        "{formatted}"
    );
    assert!(
        formatted.contains("using const Value := R(); // owned"),
        "{formatted}"
    );
    assert!(
        formatted.contains("// cleanup") && formatted.contains("// acquire"),
        "{formatted}"
    );
    let (unit, errors) = fpas_parser::parse_compilation_unit(&formatted);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(fpas_fmt::format_source(&formatted, &unit)?, formatted);
    Ok(())
}
