use super::parse_expr;
use crate::ast::*;
use crate::tests::parse_with_errors;

#[test]
fn procedure_literal_assignment() {
    match parse_expr("procedure()\n  begin\n    WriteLn(1)\n  end") {
        Expr::Closure(closure) => {
            assert!(!closure.is_function);
            assert!(closure.params.is_empty());
            assert!(closure.return_type.is_none());
        }
        other => panic!("expected procedure Closure, got {other:?}"),
    }
}

#[test]
fn function_literal_assignment() {
    match parse_expr("function(Value: integer): integer\n  begin\n    return Value + 1\n  end") {
        Expr::Closure(closure) => {
            assert!(closure.is_function);
            assert_eq!(closure.params.len(), 1);
            assert_eq!(closure.params[0].name, "Value");
            assert!(closure.return_type.is_some());
        }
        other => panic!("expected function Closure, got {other:?}"),
    }
}

#[test]
fn return_function_literal() {
    let (program, errors) = crate::parse(
        "program T;
function Make(): function(): integer;
begin
  return function(): integer
  begin
    return 1
  end
end;
begin
end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    match &program.declarations[0] {
        Decl::Function(f) => match &f.body {
            FuncBody::Block { stmts, .. } => match &stmts[0] {
                Stmt::Return(Some(Expr::Closure(closure)), _) => {
                    assert!(closure.is_function);
                }
                other => panic!("expected return Closure, got {other:?}"),
            },
        },
        other => panic!("expected function, got {other:?}"),
    }
}

#[test]
fn missing_begin_recovers() {
    let (_program, errors) =
        parse_with_errors("program T; begin var F: procedure() := procedure() end; end.");
    assert!(!errors.is_empty());
}
