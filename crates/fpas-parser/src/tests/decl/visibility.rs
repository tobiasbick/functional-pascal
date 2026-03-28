use super::*;

#[test]
fn default_visibility_is_public() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

function Greet(): integer;
begin
  return 42
end;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Public);
}

#[test]
fn explicit_public_function() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

public function Greet(): integer;
begin
  return 42
end;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Public);
}

#[test]
fn private_function() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private function Helper(): integer;
begin
  return 1
end;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
}

#[test]
fn private_procedure() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private procedure DoStuff();
begin
end;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
}

#[test]
fn private_const() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private const
  Secret: integer := 42;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
}

#[test]
fn private_var() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private var
  Internal: integer := 0;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
}

#[test]
fn private_mutable_var() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private mutable var
  Counter: integer := 0;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
}

#[test]
fn private_type() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private type
  InternalId = integer;
",
    );

    assert_eq!(unit.declarations.len(), 1);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
}

#[test]
fn mixed_visibility_declarations() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private function Helper(): integer;
begin
  return 1
end;

function PublicFn(): integer;
begin
  return Helper()
end;

public procedure ExplicitPublic();
begin
end;
",
    );

    assert_eq!(unit.declarations.len(), 3);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
    assert_eq!(unit.declarations[1].visibility(), Visibility::Public);
    assert_eq!(unit.declarations[2].visibility(), Visibility::Public);
}

#[test]
fn visibility_applies_per_block() {
    let unit = parse_unit_ok(
        "\
unit MyApp.Core;

private const
  A: integer := 1;
  B: integer := 2;

const
  C: integer := 3;
",
    );

    assert_eq!(unit.declarations.len(), 3);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Private);
    assert_eq!(unit.declarations[1].visibility(), Visibility::Private);
    assert_eq!(unit.declarations[2].visibility(), Visibility::Public);
}

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
fn private_in_program_is_accepted() {
    let program = parse_ok(
        "\
program App;

private var
  X: integer := 1;

begin
end.
",
    );

    assert_eq!(program.declarations.len(), 1);
    assert_eq!(program.declarations[0].visibility(), Visibility::Private);
}
