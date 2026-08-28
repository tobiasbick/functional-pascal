//! Integration tests for reference discovery and safe renaming.

#![allow(
    clippy::expect_used,
    reason = "navigation fixtures use explicit source offsets"
)]

mod support;

use fpas_language_service::{LanguageService, RenameError, WorkspaceContext};
use support::{TempDirectory, write_program_project};

#[test]
fn references_find_cross_unit_uses_and_optionally_include_the_declaration() {
    let temp = TempDirectory::new("references-project");
    let (manifest, main, unit) = write_program_project(&temp);
    let unit_source =
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42 end;\n";
    let main_source = "program App;\n\nuses Demo.Math;\n\nbegin\n  var A: integer := Answer();\n  var B: integer := Demo.Math.Answer();\n  var Text: string := 'Answer';\n  // Answer()\nend.\n";
    std::fs::write(&unit, unit_source).expect("write unit");
    std::fs::write(&main, main_source).expect("write program");
    let mut service = LanguageService::load(&manifest);
    let offset = main_source.find("Answer();").expect("first reference");

    let with_declaration = service
        .references(&main, offset, true)
        .expect("references with declaration")
        .value;
    assert_eq!(with_declaration.len(), 3, "{with_declaration:?}");
    assert_eq!(
        with_declaration
            .iter()
            .filter(|location| location.is_declaration)
            .count(),
        1
    );
    let without_declaration = service
        .references(&main, offset, false)
        .expect("references without declaration")
        .value;
    assert_eq!(without_declaration.len(), 2, "{without_declaration:?}");
}

#[test]
fn references_preserve_lexical_shadowing() {
    let temp = TempDirectory::new("references-shadowing");
    let source = "program Local;\n\nvar Value: integer := 1;\n\nfunction Read(Value: integer): integer;\nbegin\n  return Value\nend;\n\nbegin\n  var Result: integer := Value\nend.\n";
    let path = temp.write("local.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let parameter_use = source.find("return Value").expect("parameter reference") + 7;

    let references = service
        .references(&path, parameter_use, true)
        .expect("shadowed references")
        .value;
    assert_eq!(references.len(), 2, "{references:?}");
    assert!(references.iter().all(
        |location| location.span.offset() < source.find("end;\n\nbegin").expect("routine end")
    ));
}

#[test]
fn rename_produces_cross_unit_edits_for_declaration_and_uses() {
    let temp = TempDirectory::new("rename-project");
    let (manifest, main, unit) = write_program_project(&temp);
    let unit_source =
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42 end;\n";
    let main_source = "program App;\n\nuses Demo.Math;\n\nbegin\n  var A: integer := Answer();\n  var B: integer := Demo.Math.Answer()\nend.\n";
    std::fs::write(&unit, unit_source).expect("write unit");
    std::fs::write(&main, main_source).expect("write program");
    let mut service = LanguageService::load(&manifest);
    let offset = main_source.find("Answer();").expect("first reference");

    let edits = service
        .rename(&main, offset, "ComputeAnswer")
        .expect("project rename")
        .value;
    assert_eq!(edits.len(), 3, "{edits:?}");
    assert!(edits.iter().all(|edit| edit.new_text == "ComputeAnswer"));
    assert_eq!(edits.iter().filter(|edit| edit.path == unit).count(), 1);
    assert_eq!(edits.iter().filter(|edit| edit.path == main).count(), 2);
}

#[test]
fn reference_and_rename_spans_keep_the_snapshot_used_by_navigation() {
    let temp = TempDirectory::new("navigation-snapshot-binding");
    let (manifest, main, unit) = write_program_project(&temp);
    let unit_source =
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42 end;\n";
    let main_source =
        "program App;\n\nuses Demo.Math;\n\nbegin var Value: integer := Answer() end.\n";
    std::fs::write(&unit, unit_source).expect("write unit");
    std::fs::write(&main, main_source).expect("write program");
    let mut service = LanguageService::load(&manifest);
    let offset = main_source.find("Answer").expect("function reference");

    let references = service
        .references(&main, offset, true)
        .expect("references")
        .value;
    let edits = service
        .rename(&main, offset, "Updated")
        .expect("rename")
        .value;

    std::fs::write(&unit, format!("\n{unit_source}")).expect("change disk source after navigation");
    service.snapshot(&unit).expect("refresh disk snapshot");

    let declaration = references
        .iter()
        .find(|location| location.path == unit)
        .expect("unit declaration reference");
    assert_eq!(declaration.snapshot.source(), unit_source);
    assert_eq!(
        &declaration.snapshot.source()[declaration.span.offset()..declaration.span.end()],
        "Answer"
    );

    let declaration_edit = edits
        .iter()
        .find(|edit| edit.path == unit)
        .expect("unit declaration edit");
    assert_eq!(declaration_edit.snapshot.source(), unit_source);
    assert_eq!(
        &declaration_edit.snapshot.source()
            [declaration_edit.range.offset()..declaration_edit.range.end()],
        "Answer"
    );
}

#[test]
fn rename_rejects_keywords_and_same_scope_conflicts() {
    let temp = TempDirectory::new("rename-validation");
    let source = "program Validation;\n\nvar Value: integer := 1;\nvar Other: integer := 2;\n\nbegin\n  var Result: integer := Value\nend.\n";
    let path = temp.write("validation.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let offset = source.rfind("Value").expect("value reference");

    let keyword = service
        .rename(&path, offset, "begin")
        .expect_err("keyword must be rejected");
    assert_eq!(
        keyword,
        RenameError::InvalidIdentifier {
            name: "begin".to_owned()
        }
    );
    let conflict = service
        .rename(&path, offset, "Other")
        .expect_err("same-scope conflict must be rejected");
    assert_eq!(
        conflict,
        RenameError::Conflict {
            name: "Other".to_owned()
        }
    );
}

#[test]
fn rename_rejects_non_ascii_identifiers() {
    let temp = TempDirectory::new("rename-non-ascii");
    let source = "program Validation;\n\nvar Value: integer := 1;\n\nbegin\n  var Result: integer := Value\nend.\n";
    let path = temp.write("validation.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let offset = source.rfind("Value").expect("value reference");

    let error = service
        .rename(&path, offset, "Välue")
        .expect_err("non-ASCII identifier must be rejected");
    assert_eq!(
        error,
        RenameError::InvalidIdentifier {
            name: "Välue".to_owned()
        }
    );
}

#[test]
fn rename_rejects_compilation_units_and_dependencies_outside_the_editor_root() {
    let temp = TempDirectory::new("rename-boundary");
    let library = temp.write(
        "lib/lib.fpasprj",
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let unit = temp.write(
        "lib/src/math.fpas",
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42 end;\n",
    );
    let manifest = temp.write(
        "app/app.fpasprj",
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let main_source =
        "program App;\n\nuses Demo.Math;\n\nbegin\n  var Value: integer := Answer()\nend.\n";
    let main = temp.write("app/src/main.fpas", main_source);
    assert!(library.exists() && unit.exists());
    let mut service = LanguageService::load(&manifest);

    let unit_name = main_source.find("Math").expect("unit reference");
    assert!(
        service
            .prepare_rename(&main, unit_name)
            .expect("prepare unit rename")
            .value
            .is_none()
    );
    let function = main_source.find("Answer").expect("function reference");
    let outside = service
        .rename(&main, function, "Updated")
        .expect_err("external dependency rename must be rejected");
    assert!(matches!(outside, RenameError::OutsideWorkspace { .. }));
}

#[test]
fn rename_rejects_reverse_consumers_outside_the_editor_root() {
    let temp = TempDirectory::new("rename-reverse-consumer-boundary");
    let library = temp.write(
        "lib/lib.fpasprj",
        "[project]\nname = \"lib\"\nkind = \"library\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let unit_source =
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42 end;\n";
    let unit = temp.write("lib/src/math.fpas", unit_source);
    temp.write(
        "app/app.fpasprj",
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let main = temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Math;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    let mut service = LanguageService::load(&library);
    service
        .analyze_document(&main)
        .expect("external consumer analysis");
    let declaration = unit_source.find("Answer").expect("function declaration");

    let outside = service
        .rename(&unit, declaration, "Updated")
        .expect_err("rename must not edit an external reverse consumer");

    assert!(matches!(outside, RenameError::OutsideWorkspace { .. }));
}
