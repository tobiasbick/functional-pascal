use super::*;

#[test]
fn record_type() {
    let p = parse_ok("program T; type Point = record X: real; Y: real; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Point");
            match &td.body {
                TypeBody::Record(r) => {
                    assert_eq!(r.fields.len(), 2);
                    assert_eq!(r.fields[0].name, "X");
                    assert_eq!(r.fields[1].name, "Y");
                }
                _ => panic!("expected Record"),
            }
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_type() {
    let p = parse_ok("program T; type Color = enum Red; Green; Blue; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 3);
                assert_eq!(e.members[0].name, "Red");
                assert!(e.members[0].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_with_values() {
    let p = parse_ok("program T; type Suit = enum Hearts = 1; Diamonds = 2; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members[0].value, Some(1));
                assert_eq!(e.members[1].value, Some(2));
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_member_non_integer_value_uses_parser_code() {
    let (_, errors) =
        parse_with_errors("program T; type Suit = enum Hearts = true; end; begin end.");
    let error = errors
        .iter()
        .find_map(|diagnostic| match diagnostic {
            ParseDiagnostic::Parser(error) => Some(error),
            ParseDiagnostic::Lexer(_) => None,
        })
        .expect("expected parser diagnostic");
    assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("integer literals")),
        "expected explicit enum-value help text"
    );
}

#[test]
fn type_alias() {
    let p = parse_ok("program T; type Name = string; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Name");
            assert!(matches!(&td.body, TypeBody::Alias(_)));
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn array_type() {
    let p = parse_ok("program T; var Xs: array of integer := []; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => match &v.type_expr {
            TypeExpr::Array(inner, _) => {
                assert!(matches!(inner.as_ref(), TypeExpr::Named { .. }));
            }
            _ => panic!("expected array type"),
        },
        _ => panic!("expected Var"),
    }
}

// ── Enums with Associated Data ──────────────────────────────

#[test]
fn enum_data_single_field() {
    let p = parse_ok("program T; type Wrapper = enum Val(X: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 1);
                assert_eq!(e.members[0].name, "Val");
                assert_eq!(e.members[0].fields.len(), 1);
                assert_eq!(e.members[0].fields[0].name, "X");
                assert!(e.members[0].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_multiple_fields() {
    let p = parse_ok(
        "program T; type Shape = enum Circle(Radius: real); Rect(W: real; H: real); end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 2);
                assert_eq!(e.members[0].fields.len(), 1);
                assert_eq!(e.members[1].fields.len(), 2);
                assert_eq!(e.members[1].fields[0].name, "W");
                assert_eq!(e.members[1].fields[1].name, "H");
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_mixed_simple_and_data_variants() {
    let p = parse_ok("program T; type Token = enum Eof; Number(V: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 2);
                assert!(e.members[0].fields.is_empty());
                assert_eq!(e.members[1].fields.len(), 1);
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_fieldless_has_no_backing_value() {
    let p = parse_ok("program T; type Token = enum Eof; Number(V: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert!(e.members[0].value.is_none());
                assert!(e.members[1].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}
