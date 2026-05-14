use super::*;

#[test]
fn unit_rejects_top_level_begin_block() {
    let (_, errors) = parse_compilation_unit_with_errors(
        "\
unit MyApp.Core;
begin
end.
",
    );

    let parser_error = errors.iter().find_map(|diagnostic| match diagnostic {
        ParseDiagnostic::Parser(error) => Some(error),
        ParseDiagnostic::Lexer(_) => None,
    });
    let parser_error = parser_error.expect("expected parser diagnostic");
    assert_eq!(parser_error.code, PARSE_EXPECTED_TOKEN);
    assert!(
        parser_error
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("Unit files contain declarations only"))
    );
}

#[test]
fn unit_name_missing_segment_after_dot_keeps_placeholder_part() {
    let (unit, errors) = parse_compilation_unit_with_errors("unit MyApp.;");
    assert!(!errors.is_empty());

    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit compilation unit");
    };

    assert_eq!(unit.name.parts, vec!["MyApp", "_error_"]);
}

#[test]
fn unit_with_invalid_uses_entry_still_parses_following_declaration() {
    let (unit, errors) = parse_compilation_unit_with_errors(
        "\
unit MyApp.Core;
uses , Std.Console;

function Answer(): integer;
begin
  return 42
end;
",
    );
    assert!(!errors.is_empty());

    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit compilation unit");
    };

    assert_eq!(unit.uses.len(), 2);
    assert_eq!(unit.uses[0].parts, vec!["_error_"]);
    assert_eq!(unit.uses[1].parts, vec!["Std", "Console"]);
    assert_eq!(unit.declarations.len(), 1);
    assert!(matches!(&unit.declarations[0], Decl::Function(_)));
}

#[test]
fn unit_with_missing_uses_identifier_before_declaration_keeps_following_declaration() {
    let (unit, errors) = parse_compilation_unit_with_errors(
        "\
unit MyApp.Core;
uses function Answer(): integer;
begin
  return 42
end;
",
    );
    assert!(!errors.is_empty());

    let CompilationUnit::Unit(unit) = unit else {
        panic!("expected unit compilation unit");
    };

    assert_eq!(unit.uses.len(), 1);
    assert_eq!(unit.uses[0].parts, vec!["_error_"]);
    assert_eq!(unit.declarations.len(), 1);
    assert!(matches!(&unit.declarations[0], Decl::Function(_)));
}