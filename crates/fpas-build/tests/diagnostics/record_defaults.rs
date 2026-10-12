//! Exported defaults report unsupported values at their declaring source field.

use super::*;
use fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION;

#[test]
fn unsupported_exported_defaults_report_source_diagnostics_in_build_and_check() {
    for (declarations, field) in [
        (
            "function ReadValue(): integer; begin return 3; end function;",
            "Value: integer := ReadValue()",
        ),
        ("var Value: integer := 3;", "Value: integer := Value"),
        (
            "function ReadValue(): integer; begin return 3; end function; const Computed: integer := ReadValue();",
            "Value: integer := Computed",
        ),
        ("", "Value: integer := 1 div 0"),
        ("", "Value: integer := (9223372036854775807 + 1) div (-1)"),
        ("", "Value: real := 1 / 0"),
        ("", "Values: array of integer := [1, 2]"),
        ("", "Value: Option of integer := Some(3)"),
    ] {
        let fixture = Fixture::new();
        let source = format!(
            "unit Defaults;\n{declarations}\npublic type Settings = record\n  public {field};\nend record;\npublic type Alias = Settings;\nend unit;"
        );
        let unit = fixture.write("defaults.fpas", &source);
        let graph = build_unit_graph_for_program(
            &fixture.0.join("main.fpas"),
            std::slice::from_ref(&unit),
            &ProjectLinkMeta::default(),
        )
        .expect("parsed unit graph");
        let node = graph.get("defaults").expect("unit node");
        let selection = resolve_library_units(&graph).expect("library selection");
        for result in [
            build_library_units(&graph, &selection, &BuildOptions::default()),
            check_library_units(&graph, &selection, &BuildOptions::default()),
        ] {
            let error = result.err().expect("unsupported exported default");
            let diagnostics = error.diagnostics();
            assert_eq!(diagnostics.len(), 1, "{field}: {diagnostics:#?}");
            let record = &diagnostics[0];
            assert_eq!(
                record.diagnostic.code, SEMA_NON_CONSTANT_EXPRESSION,
                "{field}: {error}"
            );
            assert_eq!(record.path.as_deref(), Some(unit.as_path()));
            let span = record.diagnostic.span.expect("source field span");
            assert_eq!(span.source_id(), node.source_id());
            assert_eq!(span.line(), 4);
            assert!(record.diagnostic.message.contains("field"), "{error}");
            assert!(
                record
                    .diagnostic
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains("1 + 2") && help.contains("None")),
                "{error}"
            );
        }
        assert!(!unit.with_extension("fpascu").exists());
    }
}
