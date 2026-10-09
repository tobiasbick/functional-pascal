//! Task-call syntax in `docs/specs/grammar.ebnf` and `docs/pascal/language/concurrency/go.md`.

use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_parser::{Expr, PostfixOperation, Stmt, parse};

#[test]
fn formal_go_call_requires_a_direct_or_final_postfix_call() {
    let production = include_str!("../../../../docs/specs/grammar.ebnf")
        .split(';')
        .find_map(|production| {
            let (name, body) = production.rsplit_once('=')?;
            (name.lines().last()?.trim() == "go_call").then_some(body)
        })
        .expect("go_call production");
    let alternatives = production
        .split('|')
        .map(|branch| branch.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>();
    assert_eq!(
        alternatives,
        [
            "designator call_args",
            "native_array_factory",
            "primary_atom { postfix_suffix } '.' identifier call_args",
        ],
        "go_call must retain direct calls and require the final postfix suffix to be a call"
    );
}

fn task_call(target: &str, retained: bool) -> Expr {
    let statement = if retained {
        format!("const Job: task := go {target};")
    } else {
        format!("go {target};")
    };
    let source = format!("program Doc; begin {statement} end.");
    let (program, errors) = parse(&source);
    assert!(errors.is_empty(), "{source}\n{errors:#?}");
    assert_eq!(program.body.len(), 1, "{source}");
    let Some(statement) = program.body.into_iter().next() else {
        unreachable!("checked statement count");
    };
    match statement {
        Stmt::Go { expr, .. } if !retained => expr,
        Stmt::Const(binding) if retained => {
            let Expr::Go(inner, _) = binding.value else {
                unreachable!("expected retained go expression: {source}");
            };
            *inner
        }
        _ => unreachable!("expected go statement or binding: {source}"),
    }
}

#[test]
fn direct_task_call_fixtures_parse_in_both_go_forms() {
    for target in [
        "Worker()",
        "Std.Console.WriteLn('hi')",
        "Instance.Answer()",
        "Workers[0].Answer()",
        "Callable()",
        "array.Fill(2, 21)",
    ] {
        for retained in [false, true] {
            assert!(
                matches!(task_call(target, retained), Expr::Call { .. }),
                "{target}: direct call"
            );
        }
    }
}

#[test]
fn postfix_task_call_fixtures_end_in_a_method_call_in_both_go_forms() {
    for target in [
        "Make().Answer()",
        "Make().Next().Answer()",
        "MakeGroup().Items[0].Answer()",
        "MakeWorkers()[0].Answer()",
        "(Make()).Answer()",
        "Worker(Value := 42).Answer()",
        "[Make()][0].Answer()",
        "(Make() with Value := 42; end with).Answer()",
        "'answer'.Length()",
    ] {
        for retained in [false, true] {
            let Expr::Postfix { operations, .. } = task_call(target, retained) else {
                unreachable!("expected postfix call: {target}");
            };
            assert!(
                matches!(operations.last(), Some(PostfixOperation::MethodCall { .. })),
                "{target}: final method call"
            );
        }
    }
}

#[test]
fn non_call_task_targets_report_fp2005_in_both_go_forms() {
    for target in [
        "Worker",
        "42",
        "Make().Value",
        "MakeWorkers()[0]",
        "Make().Answer().Value",
        "Make().Answer()[0]",
        "Make().Answer() + 1",
        "-Make().Answer()",
        "(Make().Answer())",
        "Make() with Value := 42; end with",
    ] {
        for retained in [false, true] {
            let statement = if retained {
                format!("const Job: task := go {target};")
            } else {
                format!("go {target};")
            };
            let source = format!("program Doc; begin {statement} null; end.");
            let (program, errors) = parse(&source);
            assert_eq!(errors.len(), 1, "{source}\n{errors:#?}");
            let diagnostic = errors[0].as_diagnostic();
            assert_eq!(diagnostic.code, PARSE_EXPECTED_EXPRESSION, "{source}");
            assert!(diagnostic.message.contains("`go` requires"), "{source}");
            assert!(
                matches!(program.body.last(), Some(Stmt::Null(_))),
                "{source}"
            );
        }
    }
}
