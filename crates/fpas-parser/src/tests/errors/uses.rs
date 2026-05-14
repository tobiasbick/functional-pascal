use super::*;

#[test]
fn missing_uses_identifier_keeps_non_empty_qualified_id() {
    let (unit, errs) = parse_compilation_unit_with_errors("program T; uses ; begin end.");
    assert!(!errs.is_empty());

    let crate::CompilationUnit::Program(program) = unit else {
        panic!("expected program compilation unit");
    };

    assert_eq!(program.uses.len(), 1);
    assert!(!program.uses[0].parts.is_empty());
}

#[test]
fn missing_uses_segment_after_dot_keeps_placeholder_part() {
    let (unit, errs) = parse_compilation_unit_with_errors("program T; uses Std.; begin end.");
    assert!(!errs.is_empty());

    let crate::CompilationUnit::Program(program) = unit else {
        panic!("expected program compilation unit");
    };

    assert_eq!(program.uses.len(), 1);
    assert_eq!(program.uses[0].parts, vec!["Std", "_error_"]);
}

#[test]
fn empty_uses_entry_before_valid_unit_recovers_next_entry() {
    let (unit, errs) =
        parse_compilation_unit_with_errors("program T; uses , Std.Console; begin end.");
    assert!(!errs.is_empty());

    let crate::CompilationUnit::Program(program) = unit else {
        panic!("expected program compilation unit");
    };

    assert_eq!(program.uses.len(), 2);
    assert_eq!(program.uses[0].parts, vec!["_error_"]);
    assert_eq!(program.uses[1].parts, vec!["Std", "Console"]);
}

#[test]
fn missing_uses_identifier_before_begin_keeps_program_body() {
    let (program, errs) = parse_with_errors("program T; uses begin end.");
    assert!(!errs.is_empty());

    let parse_errors = errs
        .iter()
        .filter_map(|err| match err {
            ParseDiagnostic::Parser(diagnostic) => Some(diagnostic),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(program.body.len(), 0);
    assert_eq!(program.uses.len(), 1);
    assert_eq!(program.uses[0].parts, vec!["_error_"]);
    assert_eq!(
        parse_errors.len(),
        2,
        "unexpected parser diagnostics: {parse_errors:#?}"
    );
}

#[test]
fn missing_uses_identifier_before_declaration_keeps_following_declaration() {
    let (program, errs) = parse_with_errors(
        "program T; uses function Answer(): integer; begin return 42 end; begin end.",
    );
    assert!(!errs.is_empty());
    assert_eq!(program.uses.len(), 1);
    assert_eq!(program.uses[0].parts, vec!["_error_"]);
    assert_eq!(program.declarations.len(), 1);
    assert!(matches!(&program.declarations[0], crate::Decl::Function(_)));
}