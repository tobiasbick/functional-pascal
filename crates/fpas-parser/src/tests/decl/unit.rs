use super::*;

#[test]
fn minimal_unit() {
    let unit = parse_unit_ok("unit MyApp.Core;");
    assert_eq!(unit.name.parts, vec!["MyApp", "Core"]);
    assert!(unit.uses.is_empty());
    assert!(unit.declarations.is_empty());
}

#[test]
fn unit_with_uses_and_declarations() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Math;
uses Std.Math;

function Double(X: integer): integer;
begin
  return X * 2
end;
",
    );

    assert_eq!(unit.name.parts, vec!["MyApp", "Math"]);
    assert_eq!(unit.uses.len(), 1);
    assert_eq!(unit.uses[0].parts, vec!["Std", "Math"]);
    assert_eq!(unit.declarations.len(), 1);
    assert!(matches!(&unit.declarations[0], Decl::Function(_)));
}

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
