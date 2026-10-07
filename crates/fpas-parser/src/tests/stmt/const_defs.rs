use super::*;

#[test]
fn inline_constants_preserve_kind_initializer_and_scope() {
    let stmts = body_stmts(
        "program T; begin const X: integer := ReadValue(); begin const X: integer := 2; end; end.",
    );
    let Stmt::Const(binding) = &stmts[0] else {
        panic!("constant statement expected");
    };
    assert_eq!(binding.name, "X");
    assert!(matches!(binding.value, Expr::Call { .. }));
    assert!(matches!(&stmts[1], Stmt::Block(stmts, _) if matches!(stmts[0], Stmt::Const(_))));
}

#[test]
fn local_constants_require_their_own_keyword() {
    let (_, errors) = crate::parse("program T; begin const A: integer := 1; B: integer := 2; end.");
    assert!(
        errors
            .iter()
            .map(|error| error.as_diagnostic())
            .any(
                |error| error.code == fpas_diagnostics::codes::PARSE_MISSING_DECLARATION_KEYWORD
                    && error.message.contains("const")
            ),
        "{errors:?}"
    );
}
