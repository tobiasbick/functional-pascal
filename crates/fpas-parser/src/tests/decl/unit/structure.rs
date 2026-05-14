use super::*;

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