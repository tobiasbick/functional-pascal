use super::*;

#[test]
fn removed_members_are_rejected_and_recovery_preserves_fields_and_following_routines() {
    for member in [
        "function Get(Self: Box): integer; begin return Self.Value; end function;",
        "procedure Set(Self: Box; X: integer); begin null; end procedure;",
        "pure function Get(Self: Box): integer; begin return Self.Value; end function;",
        "static function Create(): Box; begin return Box(Value := 1); end function;",
        "static procedure Reset(); begin null; end procedure;",
        "function Map of (T)(Self: Box; F: function(X: integer): T): T; begin return F(Self.Value); end function;",
        "property Count: integer read Get write Set;",
        "property Count: integer write Set;",
        "event Changed: procedure(X: integer; Y: integer) read Get write Set;",
    ] {
        for visibility in ["", "public "] {
            let source = format!(
                "unit Model; type Box = record Value: integer; {visibility}{member} After: integer; end record; function Valid(): integer; begin return 1; end function; end unit;"
            );
            let (unit, errors) = parse_compilation_unit_with_errors(&source);
            assert!(
                errors
                    .iter()
                    .any(
                        |error| error.as_parser_error().is_some_and(|diagnostic| diagnostic
                            .message
                            .contains("Records contain stored fields")
                            && diagnostic
                                .help
                                .as_ref()
                                .is_some_and(|help| help.contains("ordinary routines")))
                    ),
                "{source}\n{errors:#?}"
            );
            let CompilationUnit::Unit(unit) = unit else {
                panic!("unit recovery");
            };
            assert_eq!(unit.declarations.len(), 2, "{source}\n{errors:#?}");
            let Decl::TypeDef(definition) = &unit.declarations[0] else {
                panic!("record recovery");
            };
            let TypeBody::Record(record) = &definition.body else {
                panic!("record recovery");
            };
            assert_eq!(
                record
                    .fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect::<Vec<_>>(),
                ["Value", "After"],
                "{source}"
            );
            assert!(
                matches!(&unit.declarations[1], Decl::Function(function) if function.name == "Valid")
            );
        }
    }
}

#[test]
fn retired_words_are_ordinary_identifiers_in_declarations_and_paths() {
    for name in [
        "static", "PROPERTY", "Event", "read", "write", "Self", "nil", "Assigned",
    ] {
        parse_ok(&format!(
            "program P; type {name} = integer; type Box = record {name}: integer := 1; end record; function Identity({name}: integer): integer; begin return {name}; end function; begin var B := Box(); B.{name} := Identity(B.{name}); end program;"
        ));
        parse_ok(&format!(
            "program P; const {name}: integer := 1; begin discard {name}; end program;"
        ));
    }
}

#[test]
fn callable_fields_and_optional_handlers_are_stored_fields() {
    let program = parse_ok(
        "program P; type Box = record Apply: function(X: integer): integer; Changed: Option of (procedure()) := Option.None; end record; begin null; end program;",
    );
    let Decl::TypeDef(definition) = &program.declarations[0] else {
        panic!("record");
    };
    let TypeBody::Record(record) = &definition.body else {
        panic!("record");
    };
    assert_eq!(record.fields.len(), 2);
    assert!(matches!(
        record.fields[0].type_expr,
        TypeExpr::FunctionType { .. }
    ));
    assert!(record.fields[1].default_value.is_some());
}
