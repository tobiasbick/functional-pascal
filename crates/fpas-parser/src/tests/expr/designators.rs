use super::parse_expr;
use crate::ast::*;

#[test]
fn simple_variable() {
    match parse_expr("X") {
        Expr::Designator(designator) => assert_eq!(designator.parts.len(), 1),
        _ => panic!("expected Designator"),
    }
}

#[test]
fn field_access() {
    match parse_expr("P.X") {
        Expr::Designator(designator) => assert_eq!(designator.parts.len(), 2),
        _ => panic!("expected Designator"),
    }
}

#[test]
fn index_access() {
    match parse_expr("Arr[0]") {
        Expr::Designator(designator) => {
            assert_eq!(designator.parts.len(), 2);
            let DesignatorPart::Ident(_, ident_span) = &designator.parts[0] else {
                panic!("expected Ident");
            };
            let DesignatorPart::Index(_, index_span) = &designator.parts[1] else {
                panic!("expected Index");
            };
            // Index span includes `[` (aligned with postfix Index spans).
            assert_eq!(index_span.offset, ident_span.offset + ident_span.length);
            assert!(index_span.length >= 3, "span={index_span:?}");
        }
        _ => panic!("expected Designator"),
    }
}

#[test]
fn chained_access() {
    match parse_expr("A.B[0].C") {
        Expr::Designator(designator) => assert_eq!(designator.parts.len(), 4),
        _ => panic!("expected Designator"),
    }
}
