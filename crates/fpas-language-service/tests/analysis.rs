#![allow(
    clippy::expect_used,
    reason = "integration fixtures use expect to keep analysis assertions focused"
)]

mod support;

use std::sync::Arc;

use fpas_language_service::{
    DocumentStore, DocumentSymbols, LanguageService, WorkspaceContext, WorkspaceSymbolIndex,
    diagnostics_for_document, format_document,
};
use support::{TempDirectory, write_program_project};

#[test]
fn loose_file_analysis_is_cached_and_formats_the_snapshot() {
    let temp = TempDirectory::new("analysis-loose");
    let path = temp.write(
        "loose.fpas",
        "program Loose; begin var Value: integer := 1 end.",
    );
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let first = service
        .analyze_document(&path)
        .expect("loose analysis must succeed");
    let second = service
        .analyze_document(&path)
        .expect("loose analysis must be cached");

    assert!(Arc::ptr_eq(&first, &second));
    assert!(first.semantic().is_some());
    assert!(diagnostics_for_document(&first).is_empty());
    assert_eq!(
        format_document(first.snapshot()).as_deref(),
        Some("program Loose;\n\nbegin\n  var Value: integer := 1\nend.\n")
    );
}

#[test]
fn project_analysis_resolves_unit_interfaces_without_writing_sidecars() {
    let temp = TempDirectory::new("analysis-project");
    let (manifest, main, unit) = write_program_project(&temp);
    let mut service = LanguageService::load(&manifest);

    let main_analysis = service
        .analyze_document(&main)
        .expect("program analysis must resolve unit interface");
    let unit_analysis = service
        .analyze_document(&unit)
        .expect("unit analysis must be available");

    assert!(main_analysis.semantic().is_some());
    assert!(unit_analysis.semantic().is_some());
    assert!(main_analysis.diagnostics().is_empty());
    assert!(unit_analysis.diagnostics().is_empty());
    assert!(
        std::fs::read_dir(temp.join("src"))
            .expect("source directory")
            .filter_map(Result::ok)
            .all(
                |entry| entry.path().extension().and_then(|value| value.to_str()) != Some("fpascu")
            )
    );
}

#[test]
fn open_unit_overlay_invalidates_cached_project_analysis_and_disk_is_unchanged() {
    let temp = TempDirectory::new("analysis-overlay");
    let (manifest, _main, unit) = write_program_project(&temp);
    let disk_source = std::fs::read_to_string(&unit).expect("disk unit source");
    let mut service = LanguageService::load(&manifest);

    let baseline = service
        .analyze_document(&unit)
        .expect("baseline unit analysis");
    service
        .documents_mut()
        .open_document(
            &unit,
            1,
            "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 'wrong'\nend;\n",
        )
        .expect("overlay opened");
    let invalid = service
        .analyze_document(&unit)
        .expect("invalid overlay still produces analysis");
    assert!(!Arc::ptr_eq(&baseline, &invalid));
    assert!(
        invalid
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.is_error())
    );

    service
        .documents_mut()
        .apply_full_text(
            &unit,
            2,
            "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 7\nend;\n",
        )
        .expect("newer overlay applied");
    let fixed = service
        .analyze_document(&unit)
        .expect("fixed overlay analysis");
    assert!(fixed.diagnostics().is_empty());
    assert!(!Arc::ptr_eq(&invalid, &fixed));
    assert_eq!(
        std::fs::read_to_string(&unit).expect("disk source remains readable"),
        disk_source
    );
}

#[test]
fn malformed_source_returns_parse_diagnostics_without_semantic_failure() {
    let temp = TempDirectory::new("analysis-malformed");
    let path = temp.write(
        "malformed.fpas",
        "program Broken;\nbegin\n  if then\nend.\n",
    );
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let analysis = service
        .analyze_document(&path)
        .expect("malformed input must produce a recoverable result");

    assert!(analysis.semantic().is_none());
    assert!(!analysis.diagnostics().is_empty());
    assert!(format_document(analysis.snapshot()).is_none());
}

#[test]
fn workspace_symbol_index_preserves_same_short_name_from_two_units() {
    let temp = TempDirectory::new("analysis-symbols");
    let first_path = temp.write(
        "first.fpas",
        "unit Demo.First;\n\npublic function Create(): integer;\nbegin\n  return 1\nend;\n",
    );
    let second_path = temp.write(
        "second.fpas",
        "unit Demo.Second;\n\npublic function Create(): integer;\nbegin\n  return 2\nend;\n",
    );
    let mut store = DocumentStore::new();
    let first = store.snapshot(&first_path).expect("first snapshot");
    let second = store.snapshot(&second_path).expect("second snapshot");
    let mut index = WorkspaceSymbolIndex::new();
    index.replace_document(&first_path, DocumentSymbols::from_snapshot(&first));
    index.replace_document(&second_path, DocumentSymbols::from_snapshot(&second));

    assert_eq!(index.find_unqualified("create").len(), 2);
    assert_eq!(index.find_qualified("Demo.First.Create").len(), 1);
    assert_eq!(index.find_qualified("Demo.Second.Create").len(), 1);
}

#[test]
fn workspace_program_analysis_resolves_workspace_library_dependency() {
    let temp = TempDirectory::new("analysis-workspace");
    let workspace = temp.write(
        "suite.fpasworkspace",
        r#"[workspace]
name = "suite"
members = ["lib/lib.fpasprj", "app/app.fpasprj"]
"#,
    );
    temp.write(
        "lib/lib.fpasprj",
        r#"[project]
name = "math"
kind = "library"

[exports]
units = ["Demo.Math"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    temp.write(
        "lib/src/math.fpas",
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    );
    temp.write(
        "app/app.fpasprj",
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
workspace = ["math"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let main = temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Math;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    let mut service = LanguageService::load(&workspace);

    let analysis = service
        .analyze_document(&main)
        .expect("workspace dependency analysis");

    assert!(analysis.semantic().is_some());
    assert!(analysis.diagnostics().is_empty());
}

#[test]
fn malformed_project_unit_returns_structured_analysis_error_for_valid_main() {
    let temp = TempDirectory::new("analysis-project-malformed-unit");
    let (manifest, main, unit) = write_program_project(&temp);
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&unit, 1, "unit Demo.Math;\npublic function Answer(")
        .expect("malformed unit overlay");

    let error = service
        .analyze_document(&main)
        .err()
        .expect("invalid dependency source must prevent project analysis");

    assert!(matches!(
        error,
        fpas_language_service::LanguageServiceError::Analysis { .. }
    ));
}
