use super::parse_expr;
use crate::ast::*;

#[test]
fn call_expr() {
    match parse_expr("Foo(1, 2)") {
        Expr::Call { args, .. } => assert_eq!(args.len(), 2),
        _ => panic!("expected Call"),
    }
}

#[test]
fn call_no_args_expr() {
    match parse_expr("Foo()") {
        Expr::Call { args, .. } => assert!(args.is_empty()),
        _ => panic!("expected Call"),
    }
}

#[test]
fn qualified_call_expr() {
    match parse_expr("Std.Math.Sqrt(4.0)") {
        Expr::Call {
            designator, args, ..
        } => {
            assert_eq!(designator.parts.len(), 3);
            assert_eq!(args.len(), 1);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn qualified_call_expr_std_unit_keyword_after_dot() {
    match parse_expr("Std.Arrays.Length(x)") {
        Expr::Call {
            designator, args, ..
        } => {
            assert_eq!(designator.parts.len(), 3);
            assert_eq!(args.len(), 1);
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn named_call_arguments_keep_written_order() {
    match parse_expr("Move(Dy := 2, Dx := 1 + 1)") {
        Expr::Call { args, .. } => {
            let names = args
                .iter()
                .map(|arg| arg.argument_name().expect("named argument"))
                .collect::<Vec<_>>();
            assert_eq!(names, ["Dy", "Dx"]);
            assert!(matches!(
                args[1].argument_value(),
                Expr::BinaryOp {
                    op: BinaryOp::Add,
                    ..
                }
            ));
        }
        _ => panic!("expected Call"),
    }
}

#[test]
fn named_arguments_parse_in_postfix_method_and_statement_calls() {
    match parse_expr("Make().Moved(Dx := 1, Dy := 2)") {
        Expr::Postfix { operations, .. } => match &operations[0] {
            PostfixOperation::MethodCall { args, .. } => {
                assert!(args.iter().all(|arg| arg.argument_name().is_some()));
            }
            _ => panic!("expected method call"),
        },
        _ => panic!("expected Postfix"),
    }
    let program = super::super::parse_ok("program T; begin Show(Count := 1, Text := 'x'); end.");
    match &program.body[0] {
        Stmt::Call { args, .. } => assert_eq!(args[1].argument_name(), Some("Text")),
        other => panic!("expected call statement, got {other:?}"),
    }
}

#[test]
fn mixed_positional_and_named_arguments_are_rejected_and_recovered() {
    let (program, errors) =
        super::super::parse_with_errors("program T; begin Move(1, Dy := 2); end.");
    let errors = errors
        .iter()
        .filter_map(crate::ParseDiagnostic::as_parser_error)
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].code,
        fpas_diagnostics::codes::PARSE_MIXED_CALL_ARGUMENTS
    );
    match &program.body[0] {
        Stmt::Call { args, .. } => {
            assert!(args.iter().all(|arg| arg.argument_name().is_none()));
        }
        other => panic!("expected call statement, got {other:?}"),
    }
}

#[test]
fn named_value_inside_ok_is_kept_for_semantic_diagnostics() {
    match parse_expr("Ok(Value := 1)") {
        Expr::ResultOk(inner, _) => assert_eq!(inner.argument_name(), Some("Value")),
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn var_arguments_and_var_parameters_parse() {
    match parse_expr("Swap(var A[I], var P.X)") {
        Expr::Call { args, .. } => {
            assert!(
                args.iter()
                    .all(|arg| matches!(arg, Expr::VarArgument { .. }))
            );
        }
        _ => panic!("expected Call"),
    }
    let program = super::super::parse_ok(
        "program T; procedure Increase(var Value: integer; Step: integer); begin end procedure; begin end.",
    );
    let Decl::Procedure(procedure) = &program.declarations[0] else {
        panic!("expected procedure");
    };
    assert_eq!(procedure.params[0].mode, ParamMode::Var);
    assert_eq!(procedure.params[1].mode, ParamMode::Value);
}

#[test]
fn record_receiver_self_cannot_be_a_var_parameter() {
    let (_, errors) = super::super::parse_with_errors(
        "program T; type Point = record X: integer; procedure Move(var Self: Point); begin end procedure; end record; begin end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
}

#[test]
fn named_var_arguments_preserve_names_designators_and_spans() {
    let source = "program T; begin Swap(B := var Items[Index()], A := var P.X); end.";
    let program = super::super::parse_ok(source);
    let Stmt::Call { args, .. } = &program.body[0] else {
        panic!("expected call statement");
    };
    for (arg, name, text) in [
        (&args[0], "B", "var Items[Index()]"),
        (&args[1], "A", "var P.X"),
    ] {
        assert_eq!(arg.argument_name(), Some(name));
        let Expr::VarArgument { designator, span } = arg.argument_value() else {
            panic!("expected var argument: {arg:?}");
        };
        assert_eq!(&source[span.offset..span.offset + span.length], text);
        assert_eq!(designator.parts.len(), 2);
        let named_span = arg.span();
        assert_eq!(
            &source[named_span.offset..named_span.offset + named_span.length],
            format!("{name} := {text}")
        );
    }
}

#[test]
fn named_var_arguments_parse_in_statement_and_postfix_calls() {
    let program = super::super::parse_ok("program T; begin Increase(Value := var Counter); end.");
    let Stmt::Call { args, .. } = &program.body[0] else {
        panic!("expected statement call");
    };
    assert!(matches!(args[0].argument_value(), Expr::VarArgument { .. }));
    let Expr::Postfix { operations, .. } = parse_expr("Make().Store(Value := var Counter)") else {
        panic!("expected postfix call");
    };
    let PostfixOperation::MethodCall { args, .. } = &operations[0] else {
        panic!("expected method call");
    };
    assert!(matches!(args[0].argument_value(), Expr::VarArgument { .. }));
}

#[test]
fn mixed_named_and_positional_var_arguments_recover_without_losing_markers() {
    for call in ["Swap(var A, B := var B)", "Swap(A := var A, var B)"] {
        let (program, errors) =
            super::super::parse_with_errors(&format!("program T; begin {call}; end."));
        let errors = errors
            .iter()
            .filter_map(crate::ParseDiagnostic::as_parser_error)
            .collect::<Vec<_>>();
        assert_eq!(errors.len(), 1, "{call}: {errors:#?}");
        assert_eq!(
            errors[0].code,
            fpas_diagnostics::codes::PARSE_MIXED_CALL_ARGUMENTS
        );
        let Stmt::Call { args, .. } = &program.body[0] else {
            panic!("expected call");
        };
        assert!(
            args.iter()
                .all(|arg| matches!(arg, Expr::VarArgument { .. }))
        );
    }
}

#[test]
fn named_var_argument_requires_a_designator() {
    for call in [
        "Increase(Value := var 42)",
        "Increase(Value := var (Counter))",
        "Increase(Value := var Counter + 1)",
    ] {
        let (_, errors) =
            super::super::parse_with_errors(&format!("program T; begin {call}; Show(1); end."));
        assert!(!errors.is_empty(), "{call}");
    }
}
