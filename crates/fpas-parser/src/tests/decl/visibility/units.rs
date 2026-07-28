use super::*;

#[test]
fn unit_declarations_default_to_private() {
    let unit = parse_unit_ok(
        "unit MyApp.Core;
         const Secret: integer := 1;
         var State: integer := 2;
         mutable var Counter: integer := 3;
         type InternalId = integer;
         function Helper(): integer; begin return 1 end;
         procedure Reset(); begin end;",
    );

    assert_eq!(unit.declarations.len(), 6);
    assert!(
        unit.declarations
            .iter()
            .all(|declaration| declaration.visibility() == Visibility::Private)
    );
}

#[test]
fn public_applies_to_every_supported_declaration_kind() {
    let unit = parse_unit_ok(
        "unit MyApp.Core;
         public const Answer: integer := 42;
         public var State: integer := 2;
         public mutable var Counter: integer := 3;
         public type PublicId = integer;
         public function Read(): integer; begin return Answer end;
         public procedure Reset(); begin end;",
    );

    assert_eq!(unit.declarations.len(), 6);
    assert!(
        unit.declarations
            .iter()
            .all(|declaration| declaration.visibility() == Visibility::Public)
    );
}

#[test]
fn public_visibility_applies_to_an_entire_declaration_block() {
    let unit = parse_unit_ok(
        "unit MyApp.Core;
         public const
           A: integer := 1;
           B: integer := 2;
         const
           C: integer := 3;",
    );

    assert_eq!(unit.declarations.len(), 3);
    assert_eq!(unit.declarations[0].visibility(), Visibility::Public);
    assert_eq!(unit.declarations[1].visibility(), Visibility::Public);
    assert_eq!(unit.declarations[2].visibility(), Visibility::Private);
}

#[test]
fn private_can_be_used_as_an_identifier() {
    let unit = parse_unit_ok(
        "unit MyApp.Core;
         function private(): integer;
         begin
           return 1
         end;",
    );

    let Decl::Function(function) = &unit.declarations[0] else {
        panic!("expected function");
    };
    assert_eq!(function.name, "private");
    assert_eq!(function.visibility, Visibility::Private);
}
