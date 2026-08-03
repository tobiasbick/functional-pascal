//! Public API regressions for source/AST identity and hostile spans.

#![allow(clippy::expect_used, clippy::panic)]

use fpas_fmt::{FormatError, format_source};
use fpas_parser::{CompilationUnit, parse_compilation_unit};

#[test]
fn matching_source_and_ast_format_successfully() {
    let source = "program T; begin end.";
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    assert!(format_source(source, &unit).is_ok());
}

#[test]
fn same_length_but_different_source_is_rejected() {
    let source = "program A; begin end.";
    let other = "program B; begin end.";
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    assert_eq!(
        format_source(other, &unit),
        Err(FormatError::SourceMismatch)
    );
}

#[test]
fn utf8_midpoint_span_is_rejected_without_panicking() {
    let source = "éprogram T; begin end.";
    let (mut unit, _) = parse_compilation_unit("program T; begin end.");
    let CompilationUnit::Program(program) = &mut unit else {
        panic!("expected program");
    };
    program.span.offset = 1;

    assert!(matches!(
        format_source(source, &unit),
        Err(FormatError::InvalidSourceSpan { offset: 1, .. })
    ));
}

#[test]
fn overflowing_span_is_rejected_without_panicking() {
    let source = "program T; begin end.";
    let (mut unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let CompilationUnit::Program(program) = &mut unit else {
        panic!("expected program");
    };
    program.span.offset = usize::MAX;
    program.span.length = usize::MAX;

    assert!(matches!(
        format_source(source, &unit),
        Err(FormatError::InvalidSourceSpan { .. })
    ));
}
