use super::*;

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
fn event_keyword_cannot_be_a_type_name() {
    let (_, errors) = parse_with_errors("program T; type Event = string; begin end.");
    assert!(!errors.is_empty());
}

#[test]
fn property_keyword_cannot_be_a_variable_name() {
    let (_, errors) = parse_with_errors("program T; var Property: integer := 1; begin end.");
    assert!(!errors.is_empty());
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

#[test]
fn channel_type() {
    let p = parse_ok("program T; var Messages: channel of string := Value; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => match &v.type_expr {
            TypeExpr::Channel(inner, _) => {
                assert!(matches!(inner.as_ref(), TypeExpr::Named { .. }));
            }
            _ => panic!("expected channel type"),
        },
        _ => panic!("expected Var"),
    }
}
