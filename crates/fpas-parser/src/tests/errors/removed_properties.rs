//! Rejection of the removed `property` record member.
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use super::parse_with_errors;
use crate::{Decl, ParseDiagnostic, TypeBody};
use fpas_diagnostics::codes::PARSE_REMOVED_PROPERTY;

fn removed_property_hints(source: &str) -> Vec<String> {
    let (_, diagnostics) = parse_with_errors(source);
    diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .filter(|error| error.code == PARSE_REMOVED_PROPERTY)
        .map(|error| error.help.clone().unwrap_or_default())
        .collect()
}

#[test]
fn property_hint_names_the_written_accessors() {
    let hints = removed_property_hints(
        "program T; type Counter = record \
         Base: integer; \
         function GetBase(Self: Counter): integer; begin return Self.Base; end function; \
         procedure SetBase(Self: Counter; Value: integer); begin end procedure; \
         property Value: integer read GetBase write SetBase; \
         end record; begin end.",
    );
    assert_eq!(hints.len(), 1, "{hints:#?}");
    assert!(hints[0].contains("`Value.GetBase()`"), "{hints:#?}");
    assert!(hints[0].contains("`Value.SetBase(NewValue)`"), "{hints:#?}");
}

#[test]
fn read_only_and_accessorless_properties_are_rejected() {
    let read_only = removed_property_hints(
        "program T; type Box = record property Width: integer read GetWidth; end record; begin end.",
    );
    assert_eq!(read_only.len(), 1, "{read_only:#?}");
    assert!(
        read_only[0].contains("`Value.GetWidth()`"),
        "{read_only:#?}"
    );
    assert!(!read_only[0].contains("write with"), "{read_only:#?}");

    let accessorless = removed_property_hints(
        "program T; type Box = record property Width: integer; end record; begin end.",
    );
    assert_eq!(accessorless.len(), 1, "{accessorless:#?}");
    assert!(
        accessorless[0].contains("declare the instance methods"),
        "{accessorless:#?}"
    );
}

#[test]
fn rejected_property_keeps_the_other_record_members() {
    let (program, diagnostics) = parse_with_errors(
        "program T; type Counter = record            Value: integer;            property Current: integer read ReadValue;            function ReadValue(Self: Counter): integer; begin return Self.Value; end function;          end record; begin end.",
    );
    assert_eq!(
        diagnostics
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .filter(|error| error.code == PARSE_REMOVED_PROPERTY)
            .count(),
        1,
        "{diagnostics:#?}"
    );
    let Decl::TypeDef(type_def) = &program.declarations[0] else {
        panic!("expected type");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected record");
    };
    assert_eq!(record.fields.len(), 1);
    assert_eq!(record.methods.len(), 1);
}

#[test]
fn property_is_an_ordinary_identifier() {
    let (_, diagnostics) = parse_with_errors(
        "program T; type Item = record property: integer; end record; \
         begin const property: integer := 1; WriteLn(property); end.",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}
