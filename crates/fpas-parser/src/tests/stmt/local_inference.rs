//! Optional statement-local annotations without changing declaration-level syntax.

use super::*;

#[test]
fn local_bindings_preserve_written_and_omitted_annotations() {
    let body =
        body_stmts("program T; begin const A := 1; var B := ['a': 2]; const C: integer := A; end.");
    let Stmt::Const(first) = &body[0] else {
        panic!("constant expected")
    };
    let Stmt::Var(second) = &body[1] else {
        panic!("variable expected")
    };
    let Stmt::Const(third) = &body[2] else {
        panic!("constant expected")
    };
    assert!(first.type_expr.is_none() && second.type_expr.is_none());
    assert!(third.type_expr.is_some());
    assert!(matches!(second.value, Expr::DictLiteral(..)));
}

#[test]
fn annotations_remain_required_outside_statement_lists() {
    for source in [
        "program T; const A := 1; begin end.",
        "program T; var A := 1; begin end.",
        "program T; function F(): integer; const A := 1; begin return A; end function; begin end.",
        "program T; function F(A): integer; begin return 1; end function; begin end.",
        "program T; type R = record A := 1; end record; begin end.",
    ] {
        let (_, errors) = crate::parse(source);
        assert!(!errors.is_empty(), "annotation required: {source}");
    }
    let (_, errors) = crate::parse_compilation_unit("unit U; public const A := 1; end unit;");
    assert!(!errors.is_empty(), "unit annotation required");
}

#[test]
fn malformed_local_annotation_recovers_the_next_binding() {
    let (program, errors) = crate::parse("program T; begin const A := ; var B := 2; end.");
    assert!(!errors.is_empty());
    assert!(program.body.iter().any(|stmt| matches!(stmt, Stmt::Var(binding) if binding.name == "B" && binding.type_expr.is_none())), "{program:?}");
}
