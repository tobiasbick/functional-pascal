use super::*;

#[test]
fn minimal_unit() {
    let unit = parse_unit_ok("unit MyApp.Core;");
    assert_eq!(unit.name.parts, vec!["MyApp", "Core"]);
    assert!(unit.uses.is_empty());
    assert!(unit.declarations.is_empty());
}

#[test]
fn single_segment_unit_name() {
    let unit = parse_unit_ok("unit Utils;");
    assert_eq!(unit.name.parts, vec!["Utils"]);
}

#[test]
fn deeply_qualified_unit_name() {
    let unit = parse_unit_ok("unit App.Sub.Module.Deep;");
    assert_eq!(unit.name.parts, vec!["App", "Sub", "Module", "Deep"]);
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
fn unit_with_multiple_uses_comma_separated() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;
uses Std.Console, Std.Math, Std.Str;
",
    );

    assert_eq!(unit.uses.len(), 3);
    assert_eq!(unit.uses[0].parts, vec!["Std", "Console"]);
    assert_eq!(unit.uses[1].parts, vec!["Std", "Math"]);
    assert_eq!(unit.uses[2].parts, vec!["Std", "Str"]);
}

#[test]
fn unit_with_only_const_and_type_declarations() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Config;

const
  MaxSize: integer := 1024;

type
  Pair = record
    A: integer;
    B: integer;
  end;
",
    );

    assert_eq!(unit.declarations.len(), 2);
    assert!(matches!(&unit.declarations[0], Decl::Const(_)));
    assert!(matches!(&unit.declarations[1], Decl::TypeDef(_)));
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
