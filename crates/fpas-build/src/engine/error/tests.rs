//! Structured failures keep source identity and native linker context.

use std::error::Error;
use std::path::Path;

use fpas_diagnostics::codes::{BUILD_ARTIFACT_IO_FAILED, LINK_INVALID_OBJECT, SEMA_TYPE_MISMATCH};
use fpas_diagnostics::{Diagnostic, DiagnosticStage, SourceSpan};
use fpas_linker::LinkError;

use super::BuildError;

#[test]
fn an_unknown_or_foreign_source_is_not_assigned_the_current_file() {
    let diagnostics = vec![
        Diagnostic::error(
            SEMA_TYPE_MISMATCH,
            "foreign source",
            None,
            SourceSpan::new_with_source(0, 1, 1, 1, 9),
        ),
        Diagnostic::error_without_source(SEMA_TYPE_MISMATCH, "no location", None),
    ];
    let failure =
        BuildError::from_diagnostics(diagnostics.clone(), Some((3, Path::new("unit.fpas"))));
    for (record, original) in failure.diagnostics().iter().zip(diagnostics) {
        assert_eq!(record.diagnostic, original);
        assert_eq!(record.path, None);
    }
    assert!(!failure.to_string().contains("unit.fpas"));
}

#[test]
fn linking_preserves_the_native_error_and_error_chain() {
    let (program, diagnostics) = fpas_parser::parse(r#"program Demo; begin null; end program;"#);
    assert!(diagnostics.is_empty());
    let object = fpas_compiler::compile_program_object_with_support(&program, &[], &[])
        .expect("valid object");
    let units = crate::BuiltUnits {
        objects: vec![object],
        interfaces: Default::default(),
        events: Vec::new(),
        linked_units: Vec::new(),
        supporting_interfaces: Vec::new(),
    };
    let failure = crate::engine::link_program(units, &program, None)
        .err()
        .expect("a dependency must not have a program entry");
    let native = failure.link_error().expect("typed link failure");
    assert!(matches!(native, LinkError::UnitEntry(_)));
    assert_eq!(
        failure.to_string(),
        format!("error[{LINK_INVALID_OBJECT}]: {native}")
    );
    assert_eq!(
        failure
            .source()
            .and_then(|error| error.downcast_ref::<LinkError>()),
        Some(native)
    );
    let [record] = failure.diagnostics() else {
        panic!("a link failure has exactly one record");
    };
    assert_eq!(record.diagnostic.code, LINK_INVALID_OBJECT);
    assert_eq!(record.diagnostic.stage(), DiagnosticStage::Project);
    assert_eq!(record.diagnostic.span, None);
}

#[test]
fn coded_build_failures_render_their_code_help_and_source() {
    let failure = BuildError::new(BUILD_ARTIFACT_IO_FAILED, "cannot publish artifact")
        .with_help("Check the output directory.")
        .in_source(Path::new("main.fpas"));
    assert_eq!(
        failure.to_string(),
        "main.fpas: error[F5024]: cannot publish artifact\n  help: Check the output directory."
    );
    let [record] = failure.diagnostics() else {
        panic!("one record");
    };
    assert_eq!(record.diagnostic.code, BUILD_ARTIFACT_IO_FAILED);
    assert_eq!(record.path.as_deref(), Some(Path::new("main.fpas")));
    assert!(failure.link_error().is_none());
    assert!(failure.source().is_none());
}
