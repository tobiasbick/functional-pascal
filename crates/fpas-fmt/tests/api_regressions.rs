//! Public API regressions for source/AST identity and hostile spans.

#![allow(clippy::expect_used, clippy::panic)]

use fpas_fmt::{FormatError, format_source};
use fpas_parser::{CompilationUnit, parse_compilation_unit};

#[test]
fn matching_source_and_ast_format_successfully() {
    let source = r#"program T; begin null; end program;"#;
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    assert!(format_source(source, &unit).is_ok());
}

#[test]
fn same_length_but_different_source_is_rejected() {
    let source = r#"program A; begin null; end program;"#;
    let other = r#"program B; begin null; end program;"#;
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
    let (mut unit, _) = parse_compilation_unit(r#"program T; begin null; end program;"#);
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
    let source = r#"program T; begin null; end program;"#;
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
