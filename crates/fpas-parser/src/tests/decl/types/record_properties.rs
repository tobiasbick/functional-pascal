use super::*;

#[test]
fn record_read_write_property() {
    let p = parse_ok(
        "program T; type Button = record \
         function GetText(Self: Button): string; begin return '' end; \
         procedure SetText(Self: Button; Value: string); begin end; \
         property Text: string read GetText write SetText; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.properties.len(), 1);
                let prop = &r.properties[0];
                assert_eq!(prop.name, "Text");
                assert_eq!(prop.read.as_deref(), Some("GetText"));
                assert_eq!(prop.write.as_deref(), Some("SetText"));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_read_only_property() {
    let p = parse_ok(
        "program T; type Box = record \
         function GetWidth(Self: Box): integer; begin return 0 end; \
         property Width: integer read GetWidth; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.properties.len(), 1);
                assert_eq!(r.properties[0].read.as_deref(), Some("GetWidth"));
                assert!(r.properties[0].write.is_none());
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_write_only_property() {
    let p = parse_ok(
        "program T; type Box = record \
         procedure SetPassword(Self: Box; Value: string); begin end; \
         property Password: string write SetPassword; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.properties.len(), 1);
                assert!(r.properties[0].read.is_none());
                assert_eq!(r.properties[0].write.as_deref(), Some("SetPassword"));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn property_rejects_unknown_accessor_keyword() {
    let (_, errors) = parse_with_errors(
        "program T; type Box = record \
         property Width: integer foo GetWidth; \
         end; begin end.",
    );
    assert!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|e| { e.message.contains("Expected `read` or `write`") }),
        "{errors:#?}"
    );
}

#[test]
fn property_without_accessors_is_rejected() {
    let (_, errors) = parse_with_errors(
        "program T; type Box = record \
         property Width: integer; \
         end; begin end.",
    );
    assert!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|e| e.message.contains("at least one of `read` or `write`")),
        "{errors:#?}"
    );
}

#[test]
fn property_keeps_first_read_on_duplicate() {
    let (program, errors) = parse_with_errors(
        "program T; type Box = record \
         function GetA(Self: Box): integer; begin return 0 end; \
         function GetB(Self: Box): integer; begin return 1 end; \
         property Width: integer read GetA read GetB; \
         end; begin end.",
    );
    assert!(
        errors
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .any(|e| e.message.contains("Duplicate `read`")),
        "{errors:#?}"
    );
    match &program.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.properties[0].read.as_deref(), Some("GetA"));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}
