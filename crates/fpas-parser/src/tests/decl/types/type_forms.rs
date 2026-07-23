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
fn soft_keyword_event_can_be_a_type_name() {
    let p = parse_ok("program T; type Event = string; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Event");
            assert!(matches!(&td.body, TypeBody::Alias(_)));
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn soft_keyword_property_can_be_a_var_name() {
    let p = parse_ok("program T; var Property: integer := 1; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => assert_eq!(v.name, "Property"),
        _ => panic!("expected Var"),
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
