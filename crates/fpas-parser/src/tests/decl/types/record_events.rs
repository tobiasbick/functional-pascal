use super::*;

#[test]
fn record_events_default_to_private_and_accept_public() {
    let unit = parse_unit_ok(
        "unit Demo.Types; \
         type Counter = record \
           event Hidden: procedure() read GetHidden write SetHidden; \
           public event Changed: procedure() read GetChanged write SetChanged; \
         end;",
    );
    let Decl::TypeDef(type_def) = &unit.declarations[0] else {
        panic!("expected type");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected record");
    };
    assert_eq!(record.events[0].visibility, Visibility::Private);
    assert_eq!(record.events[1].visibility, Visibility::Public);
}

#[test]
fn record_event_parses() {
    let p = parse_ok(
        "program T; type Button = record \
         function ReadOnClick(Self: Button): Option of procedure(); begin return None end; \
         procedure WriteOnClick(Self: Button; H: Option of procedure()); begin end; \
         event OnClick: procedure() read ReadOnClick write WriteOnClick; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.events.len(), 1);
                let event = &r.events[0];
                assert_eq!(event.name, "OnClick");
                assert_eq!(event.read, "ReadOnClick");
                assert_eq!(event.write, "WriteOnClick");
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn nil_literal_parses() {
    let (_, errors) = parse_with_errors("program T; begin X := nil end.");
    assert!(
        errors.is_empty()
            || errors
                .iter()
                .filter_map(ParseDiagnostic::as_parser_error)
                .all(|e| !e.message.contains("Expected expression")),
        "{errors:#?}"
    );
}

#[test]
fn event_without_accessors_is_rejected() {
    let (program, errors) = parse_with_errors(
        "program T; type Button = record \
         event OnClick: procedure(); \
         end; begin end.",
    );
    assert!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|e| e.message.contains("requires both `read` and `write`")),
        "{errors:#?}"
    );
    match &program.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert!(r.events[0].read.is_empty());
                assert!(r.events[0].write.is_empty());
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn event_with_only_read_is_rejected() {
    let (_, errors) = parse_with_errors(
        "program T; type Button = record \
         function ReadOnClick(Self: Button): Option of procedure(); begin return None end; \
         event OnClick: procedure() read ReadOnClick; \
         end; begin end.",
    );
    assert!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|e| e.message.contains("requires both `read` and `write`")),
        "{errors:#?}"
    );
}
