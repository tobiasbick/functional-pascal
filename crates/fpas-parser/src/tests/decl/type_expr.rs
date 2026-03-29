use super::*;

#[test]
fn function_type_expr() {
    let p = parse_ok("program T; var F: function(X: integer): integer := Add; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => {
            assert!(matches!(v.type_expr, TypeExpr::FunctionType { .. }));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn procedure_type_expr() {
    let p = parse_ok("program T; var P: procedure(X: integer) := DoStuff; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => {
            assert!(matches!(v.type_expr, TypeExpr::ProcedureType { .. }));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn ref_type_expr() {
    let p = parse_ok("program T; var NodeRef: ref integer := new integer with end; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => {
            assert!(matches!(v.type_expr, TypeExpr::Ref { .. }));
        }
        _ => panic!("expected Var"),
    }
}
