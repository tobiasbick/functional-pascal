//! Required statement terminators and recovery at block boundaries.

use super::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

#[test]
fn missing_final_terminators_are_diagnosed_at_each_boundary() {
    for (source, boundary) in [
        ("program P; begin WriteLn('x') end.", "end"),
        (
            "program P; procedure F(); begin return end procedure; begin end.",
            "end",
        ),
        ("program P; begin if C then A() else B(); end.", "else"),
        ("program P; begin while C do A() end.", "end"),
        (
            "program P; begin for I: integer := 1 to 2 do A() end.",
            "end",
        ),
        (
            "program P; begin for I: integer in Values do A() end.",
            "end",
        ),
        ("program P; begin repeat A() until C; end.", "until"),
        ("program P; begin repeat A(); until C end.", "end"),
        ("program P; begin case V of 1: A() end; end.", "end"),
        (
            "program P; begin case V of 1: A(); else B() end; end.",
            "end",
        ),
        ("program P; begin if C then A(); else B() end.", "end"),
        (
            "program P; begin Consume(function(): integer begin return 1 end); end.",
            "end",
        ),
    ] {
        let (_, diagnostics) = parse_with_errors(source);
        let error = diagnostics
            .iter()
            .filter_map(ParseDiagnostic::as_parser_error)
            .find(|error| error.message.starts_with("Expected `;` after statement"))
            .unwrap_or_else(|| panic!("{source}: {diagnostics:#?}"));
        assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
        assert_eq!(error.expected.as_deref(), Some(";"));
        assert_eq!(error.found.as_deref(), Some(boundary));
        let span = error.span.expect("terminator diagnostic span");
        assert_eq!(&source[span.offset()..span.end()], boundary, "{source}");
        assert!(
            error
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("Terminate"))
        );
    }
}

#[test]
fn single_statement_control_bodies_share_their_last_terminator() {
    for source in [
        "program P; begin if C then A(); else B(); end.",
        "program P; begin if C then A(); end.",
        "program P; begin while C do A(); end.",
        "program P; begin for I: integer := 1 to 2 do A(); end.",
        "program P; begin for I: integer in Values do A(); end.",
        "program P; begin if C then while D do A(); else B(); end.",
        "program P; begin if C then begin if D then A(); end; else B(); end.",
        "program P; begin case V of 1: A(); 2: B(); else C(); end; end.",
        "program P; begin repeat A(); until C; end.",
    ] {
        let (_, diagnostics) = parse_with_errors(source);
        assert!(diagnostics.is_empty(), "{source}: {diagnostics:#?}");
    }
}

#[test]
fn terminator_recovery_preserves_the_next_statement() {
    let source = "program P; begin A() B(); end.";
    let (program, diagnostics) = parse_with_errors(source);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(program.body.len(), 2, "{:#?}", program.body);
}

#[test]
fn comments_do_not_replace_terminators_or_change_else_ownership() {
    let source = "program P; begin if C then A(); // then\nelse B(); // else\nend.";
    let (program, diagnostics) = parse_with_errors(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert!(matches!(
        program.body.as_slice(),
        [crate::Stmt::If {
            else_branch: Some(_),
            ..
        }]
    ));
    let (_, diagnostics) = parse_with_errors("program P; begin A() // missing\nend.");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");

    let (program, diagnostics) =
        parse_with_errors("program P; begin if C then if D then A(); else B(); end.");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let [
        crate::Stmt::If {
            then_branch,
            else_branch: None,
            ..
        },
    ] = program.body.as_slice()
    else {
        panic!("outer if must have no else: {:#?}", program.body);
    };
    assert!(matches!(
        then_branch.as_ref(),
        crate::Stmt::If {
            else_branch: Some(_),
            ..
        }
    ));
}

#[test]
fn empty_blocks_remain_valid_but_extra_terminators_are_rejected() {
    let (_, diagnostics) = parse_with_errors("program P; begin begin end; end.");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let (_, diagnostics) = parse_with_errors("program P; begin A();; end.");
    assert!(!diagnostics.is_empty());
}

#[test]
fn declarations_keep_their_required_terminators() {
    for source in [
        "program P; const X: integer := 1 begin end.",
        "program P; type X = integer begin end.",
        "program P; procedure F(); begin end procedure begin end.",
        "program P; type R = record X: integer end record; begin end.",
        "program P; type E = enum A end enum; begin end.",
    ] {
        let (_, diagnostics) = parse_with_errors(source);
        assert!(
            diagnostics
                .iter()
                .filter_map(ParseDiagnostic::as_parser_error)
                .any(|error| error.code == PARSE_EXPECTED_TOKEN && error.message.contains("`;`")),
            "{source}: {diagnostics:#?}"
        );
    }
}
