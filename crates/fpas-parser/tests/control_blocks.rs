//! AP13.4 control bodies, named endings, diagnostics, and recovery.

#![allow(
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
fn conditional_lists_distinguish_elsif_from_nested_else_if() {
    let program = parse_ok(
        "if true then A(); B(); elsif false then C(); else if true then D(); end if; E(); end if;",
    );
    let Stmt::If {
        then_branch,
        else_branch,
        ..
    } = &program.body[0]
    else {
        panic!("if")
    };
    assert!(matches!(then_branch.as_ref(), Stmt::Block(stmts, _) if stmts.len() == 2));
    let Some(Stmt::If { else_branch, .. }) = else_branch.as_deref() else {
        panic!("elsif")
    };
    let Some(Stmt::Block(stmts, _)) = else_branch.as_deref() else {
        panic!("else list")
    };
    assert_eq!(stmts.len(), 2);
    assert!(matches!(stmts[0], Stmt::If { .. }));
}

#[test]
fn all_loop_forms_accept_multiple_statements_and_nested_scopes() {
    parse_ok(
        "for I: integer := 1 to 3 do A(); B(); end for;
        for I: integer := 3 downto 1 do null; end for;
        for X: integer in Items do begin const Y: integer := X; A(Y); end; B(); end for;
        while true do if false then null; end if; break; end while;
        repeat null; until true;",
    );
    parse_ok(
        "IF true THEN NULL; ELSIF false THEN NULL; ELSE NULL; END IF;
        WHILE false DO NULL; END WHILE; FOR I: integer := 1 TO 2 DO NULL; END FOR;",
    );
}

#[test]
fn empty_control_bodies_require_an_explicit_no_op() {
    for body in [
        "if true then end if;",
        "if true then null; elsif false then end if;",
        "if true then null; else end if;",
        "for I: integer := 1 to 2 do end for;",
        "for X: integer in Items do end for;",
        "while true do end while;",
        "repeat until true;",
    ] {
        let (_, errors) = parse(&format!("program T; begin {body} end."));
        assert!(
            errors.iter().any(|error| error
                .as_diagnostic()
                .help
                .as_deref()
                .is_some_and(|help| help.contains("null;"))),
            "{body}: {errors:#?}"
        );
    }
    parse_ok("begin end; null;");
    parse_ok("");
}

#[test]
fn every_statement_and_control_ending_requires_a_terminator() {
    for body in [
        "if true then null end if;",
        "if true then A() elsif false then null; end if;",
        "if true then null; else A() end if;",
        "if true then null; end if B();",
        "while false do null; end while B();",
        "for I: integer := 1 to 2 do null; end for B();",
        "repeat null until true;",
    ] {
        let (_, errors) = parse(&format!("program T; begin {body} end."));
        assert!(
            errors
                .iter()
                .any(|error| error.as_diagnostic().expected.as_deref() == Some(";")),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn bare_and_mismatched_endings_name_the_canonical_replacement() {
    for (body, expected, found) in [
        ("if true then null; end;", "end if;", "end;"),
        ("while false do null; end;", "end while;", "end;"),
        ("for I: integer := 1 to 2 do null; end;", "end for;", "end;"),
        ("if true then null; end while;", "end if;", "end while;"),
    ] {
        let (program, errors) = parse(&format!("program T; begin {body} After(); end."));
        assert_eq!(program.body.len(), 2, "{errors:#?}");
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
fn missing_inner_end_preserves_the_outer_loop_and_declaration() {
    let source = "program T; procedure P(); begin while true do if false then null; end while; return; end procedure; begin P(); end.";
    let (program, errors) = parse(source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].as_diagnostic().expected.as_deref(),
        Some("end if;")
    );
    assert_eq!(
        errors[0].as_diagnostic().found.as_deref(),
        Some("end while;")
    );
    let Decl::Procedure(procedure) = &program.declarations[0] else {
        panic!("procedure")
    };
    let FuncBody::Block { stmts, .. } = &procedure.body;
    assert_eq!(stmts.len(), 2);
    assert_eq!(program.body.len(), 1);
}

#[test]
fn unclosed_else_if_suggests_elsif() {
    let (_, errors) =
        parse("program T; begin if true then null; else if false then null; end if; end.");
    assert!(
        errors.iter().any(|error| error
            .as_diagnostic()
            .help
            .as_deref()
            .is_some_and(|help| help.contains("elsif"))),
        "{errors:#?}"
    );
}

#[test]
fn explicit_scoping_end_does_not_close_the_conditional() {
    parse_ok("if true then begin const X: integer := 1; A(X); end; B(); end if;");
    let (_, errors) = parse("program T; begin if true then begin null; end; end.");
    assert!(
        errors
            .iter()
            .any(|error| error.as_diagnostic().expected.as_deref() == Some("end if;"))
    );
}

#[test]
fn long_elsif_chains_respect_the_parser_nesting_budget() {
    let source = format!(
        "program T; begin if true then null; {} end if; end.",
        "elsif false then null; ".repeat(300)
    );
    let (_, errors) = parse(&source);
    assert!(
        errors
            .iter()
            .any(|error| error.as_diagnostic().message.contains("nesting limit")),
        "{errors:#?}"
    );
}
