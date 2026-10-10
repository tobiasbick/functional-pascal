//! Ordinary identifiers in record fields and routines.
//!
//! **Documentation:** `docs/pascal/getting-started/keywords.md`

use super::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn ordinary_names_parse_as_fields_and_routines() {
    for name in ["event", "nil", "read", "write", "Assigned"] {
        let source = format!(
            "program T; type Holder = record
               {name}: integer;
             end record;
             function {name}(Value: integer): integer;
             begin return Value; end function;
             begin const H: Holder := Holder({name} := {name}(1)); end."
        );
        let program = parse_ok(&source);
        let Decl::TypeDef(definition) = &program.declarations[0] else {
            panic!("expected record type");
        };
        let TypeBody::Record(record) = &definition.body else {
            panic!("expected record body");
        };
        assert_eq!(record.fields.len(), 1);
        assert_eq!(record.fields[0].name, name);
    }
}

#[test]
fn invalid_member_declarations_use_ordinary_parser_errors() {
    for member in [
        "event OnClick: procedure();",
        "event OnClick: procedure() read Get write Set;",
        "EvEnT OnClick: procedure() write Set read Get;",
    ] {
        let source = format!("program T; type Holder = record {member} end record; begin end.");
        let (_, errors) = parse_with_errors(&source);
        assert!(!errors.is_empty(), "{source}");
        assert_eq!(
            errors[0].as_diagnostic().code,
            PARSE_EXPECTED_TOKEN,
            "{errors:#?}"
        );
        assert!(
            errors.iter().all(|error| {
                let diagnostic = error.as_diagnostic();
                !diagnostic.message.contains("event")
                    && !diagnostic
                        .help
                        .as_deref()
                        .is_some_and(|hint| hint.contains("event"))
            }),
            "{errors:#?}"
        );
    }
}
