use super::*;

#[test]
fn program_declarations_default_to_public() {
    let program = parse_ok(
        "\
program App;

var
  X: integer := 1;

begin
end.
",
    );

    assert_eq!(program.declarations.len(), 1);
    assert_eq!(program.declarations[0].visibility(), Visibility::Public);
}

#[test]
fn private_in_program_is_rejected() {
    let (program, errors) = parse_with_errors(
        "\
program App;

private var
  X: integer := 1;

begin
end.
",
    );

    assert_eq!(program.declarations.len(), 1);
    assert_eq!(program.declarations[0].visibility(), Visibility::Public);
    let parser_error = errors.iter().find_map(|diagnostic| match diagnostic {
        ParseDiagnostic::Parser(error) => Some(error),
        ParseDiagnostic::Lexer(_) => None,
    });
    let parser_error = parser_error.expect("expected parser diagnostic");
    assert_eq!(parser_error.code, PARSE_INVALID_VISIBILITY);
    assert!(
        parser_error
            .message
            .contains("`private` is not valid in a `program` file"),
        "{parser_error:#?}"
    );
}

#[test]
fn public_in_program_is_rejected() {
    let (program, errors) = parse_with_errors(
        "\
program App;

public var
  X: integer := 1;

begin
end.
",
    );

    assert_eq!(program.declarations.len(), 1);
    assert_eq!(program.declarations[0].visibility(), Visibility::Public);
    let parser_error = errors.iter().find_map(|diagnostic| match diagnostic {
        ParseDiagnostic::Parser(error) => Some(error),
        ParseDiagnostic::Lexer(_) => None,
    });
    let parser_error = parser_error.expect("expected parser diagnostic");
    assert_eq!(parser_error.code, PARSE_INVALID_VISIBILITY);
    assert!(
        parser_error
            .message
            .contains("`public` is not valid in a `program` file"),
        "{parser_error:#?}"
    );
}
