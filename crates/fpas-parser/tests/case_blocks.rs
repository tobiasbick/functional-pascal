//! AP13.5 arm lists, named endings, diagnostics, and boundary recovery.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

use fpas_parser::{Decl, FuncBody, Program, Stmt, parse};

fn parse_ok(body: &str) -> Program {
    let source = format!("program T; begin {body} end.");
    let (program, errors) = parse(&source);
    assert!(errors.is_empty(), "{source}\n{errors:#?}");
    program
}

#[test]
fn arms_and_catch_all_accept_scoped_statement_lists() {
    let program = parse_ok(
        "case Value of when 1, 2: A(); B(); when 3..5 if Ready: null; else C(); D(); end case;",
    );
    let Stmt::Case {
        arms, else_body, ..
    } = &program.body[0]
    else {
        panic!("case")
    };
    assert_eq!(arms.len(), 2);
    assert_eq!(arms[0].labels.len(), 2);
    assert!(arms[1].guard.is_some());
    assert!(matches!(&arms[0].body, Stmt::Block(stmts, _) if stmts.len() == 2));
    assert_eq!(else_body.as_ref().expect("catch-all").len(), 2);
}

#[test]
fn nested_cases_and_controls_keep_arm_and_else_ownership() {
    parse_ok(
        "case X of when 1: case Y of when Some(const V) if V > 0: A(V); when None: null; when Some(_): B(); end case; C(); when 2: if true then A(); else B(); end if; while false do null; end while; else D(); end case;",
    );
    parse_ok(
        "CASE X OF WHEN Shape.Circle(R) IF R > 0: NULL; WHEN Shape.Point: NULL; WHEN Shape.Circle(_): NULL; END CASE;",
    );
}

#[test]
fn empty_cases_and_else_only_cases_are_rejected() {
    for body in ["case X of end case;", "case X of else null; end case;"] {
        let (_, errors) = parse(&format!("program T; begin {body} end."));
        assert!(
            errors.iter().any(|error| error
                .as_diagnostic()
                .message
                .contains("at least one case arm")),
            "{errors:#?}"
        );
    }
}

#[test]
fn empty_arm_lists_require_null() {
    for body in [
        "case X of when 1: end case;",
        "case X of when 1: when 2: null; end case;",
        "case X of when 1: null; else end case;",
    ] {
        let (_, errors) = parse(&format!("program T; begin {body} end."));
        assert!(
            errors.iter().any(|error| error
                .as_diagnostic()
                .help
                .as_deref()
                .is_some_and(|help| help.contains("null;"))),
            "{errors:#?}"
        );
    }
    parse_ok("case X of when 1: null; else null; end case;");
}

#[test]
fn statements_and_named_endings_require_semicolons() {
    for body in [
        "case X of when 1: A() when 2: null; end case;",
        "case X of when 1: A() else null; end case;",
        "case X of when 1: null; else A() end case;",
        "case X of when 1: null; end case B();",
    ] {
        let (_, errors) = parse(&format!("program T; begin {body} end."));
        assert!(
            errors
                .iter()
                .any(|error| error.as_diagnostic().expected.as_deref() == Some(";")),
            "{errors:#?}"
        );
    }
}

#[test]
fn legacy_arms_and_wrong_endings_show_the_replacement() {
    for (body, expected, found) in [
        ("case X of 1: null; end case;", "when", "1"),
        ("case X of when 1: null; end;", "end case;", "end;"),
        ("case X of when 1: null; end if;", "end case;", "end if;"),
    ] {
        let (_, errors) = parse(&format!("program T; begin {body} After(); end."));
        assert!(
            errors.iter().any(|error| {
                let error = error.as_diagnostic();
                error.expected.as_deref() == Some(expected) && error.found.as_deref() == Some(found)
            }),
            "{errors:#?}"
        );
    }
}

#[test]
fn missing_conditional_ending_preserves_the_next_when_arm() {
    let (program, errors) = parse(
        "program T; begin case X of when 1: if true then null; when 2: null; end case; After(); end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].as_diagnostic().expected.as_deref(),
        Some("end if;")
    );
    let Stmt::Case { arms, .. } = &program.body[0] else {
        panic!("case")
    };
    assert_eq!(arms.len(), 2);
    assert_eq!(program.body.len(), 2);
}

#[test]
fn missing_case_ending_preserves_the_enclosing_declaration() {
    let (program, errors) = parse(
        "program T; procedure P(); begin case X of when 1: null; end procedure; begin P(); end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].as_diagnostic().expected.as_deref(),
        Some("end case;")
    );
    assert_eq!(
        errors[0].as_diagnostic().found.as_deref(),
        Some("end procedure;")
    );
    let Decl::Procedure(procedure) = &program.declarations[0] else {
        panic!("procedure")
    };
    let FuncBody::Block { stmts, .. } = &procedure.body;
    assert_eq!(stmts.len(), 1);
    assert_eq!(program.body.len(), 1);
}

#[test]
fn explicit_compound_blocks_remain_inside_the_arm_scope() {
    let program = parse_ok(
        "case X of when 1: begin const Local: integer := 1; A(Local); end; B(); end case;",
    );
    let Stmt::Case { arms, .. } = &program.body[0] else {
        panic!("case")
    };
    let Stmt::Block(stmts, _) = &arms[0].body else {
        panic!("arm list")
    };
    assert_eq!(stmts.len(), 2);
    assert!(matches!(stmts[0], Stmt::Block(..)));
}
