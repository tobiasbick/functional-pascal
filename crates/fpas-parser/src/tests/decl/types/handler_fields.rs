//! Optional handlers parse as ordinary fields with visibility and defaults.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::*;

#[test]
fn handler_fields_default_to_private_and_accept_public() {
    let unit = parse_unit_ok(
        "unit Demo.Types;
         type Counter = record
           Hidden: Option of procedure() := None;
           public Changed: Option of procedure() := None;
         end record; end unit;",
    );
    let Decl::TypeDef(type_def) = &unit.declarations[0] else {
        panic!("expected type");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected record");
    };
    assert_eq!(record.fields[0].visibility, Visibility::Private);
    assert_eq!(record.fields[1].visibility, Visibility::Public);
}

#[test]
fn handler_field_and_parenthesized_none_parse() {
    let program = parse_ok(
        "program T; type Button = record
           OnClick: Option of procedure() := None;
         end record;
         begin var B: Button := Button(); B.OnClick := (None); end.",
    );
    let Decl::TypeDef(type_def) = &program.declarations[0] else {
        panic!("expected type");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected record");
    };
    assert_eq!(record.fields.len(), 1);
    assert_eq!(record.fields[0].name, "OnClick");
    assert!(record.fields[0].default_value.is_some());
}
