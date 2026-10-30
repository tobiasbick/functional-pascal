//! Integration tests for reference discovery and safe renaming.

#![allow(
    clippy::expect_used,
    reason = "navigation fixtures use explicit source offsets"
)]

use crate::support;

use fpas_language_service::{LanguageService, RenameError, WorkspaceContext};
use support::{TempDirectory, write_program_project};

#[test]
fn references_find_cross_unit_uses_and_optionally_include_the_declaration() {
    let temp = TempDirectory::new("references-project");
    let (manifest, main, unit) = write_program_project(&temp);
    let unit_source = "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42; end function;\nend unit;\n";
    let main_source = "program App;\n\nuses Demo.Math as Math;\n\nbegin\n  const A: integer := Math.Answer();\n  const B: integer := Math.Answer();\n  const Text: string := 'Answer';\n  // Answer()\nend.\n";
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
    let source = "program Local;\n\nconst Value: integer := 1;\n\nfunction ReadValue(Value: integer): integer;\nbegin\n  return Value;\nend function;\n\nbegin\n  const Result: integer := Value;\nend.\n";
    let path = temp.write("local.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let parameter_use = source.find("return Value").expect("parameter reference") + 7;

    let references = service
        .references(&path, parameter_use, true)
        .expect("shadowed references")
        .value;
    assert_eq!(references.len(), 2, "{references:?}");
    assert!(references.iter().all(|location| location.span.offset()
        < source.find("end function;\n\nbegin").expect("routine end")));
}

#[test]
fn rename_produces_cross_unit_edits_for_declaration_and_uses() {
    let temp = TempDirectory::new("rename-project");
    let (manifest, main, unit) = write_program_project(&temp);
    let unit_source = "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42; end function;\nend unit;\n";
    let main_source = "program App;\n\nuses Demo.Math as Math;\n\nbegin\n  const A: integer := Math.Answer();\n  const B: integer := Math.Answer();\nend.\n";
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
    let unit_source = "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42; end function;\nend unit;\n";
    let main_source = "program App;\n\nuses Demo.Math as Math;\n\nbegin const Value: integer := Math.Answer(); end.\n";
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
    let source = "program Validation;\n\nconst Value: integer := 1;\nconst Other: integer := 2;\n\nbegin\n  const Result: integer := Value;\nend.\n";
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
    let source = "program Validation;\n\nconst Value: integer := 1;\n\nbegin\n  const Result: integer := Value;\nend.\n";
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
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42; end function;\nend unit;\n",
    );
    let manifest = temp.write(
        "app/app.fpasprj",
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let main_source = "program App;\n\nuses Demo.Math as Math;\n\nbegin\n  const Value: integer := Math.Answer();\nend.\n";
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
    let unit_source = "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin return 42; end function;\nend unit;\n";
    let unit = temp.write("lib/src/math.fpas", unit_source);
    temp.write(
        "app/app.fpasprj",
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nprojects = [\"../lib/lib.fpasprj\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let main = temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Math as Math;\n\nbegin\n  const Value: integer := Math.Answer();\nend.\n",
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

#[test]
fn named_argument_labels_follow_parameters_not_same_named_locals() {
    let temp = TempDirectory::new("references-named-arguments");
    let source = "program Named;\n\nfunction Sub(Left: integer; Right: integer): integer;\nbegin\n  return Left - Right;\nend function;\n\nbegin\n  const Left: integer := 10;\n  const A: integer := Sub(Right := 1, Left := Left);\nend.\n";
    let path = temp.write("named.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let parameter = source
        .find("Left: integer;")
        .expect("parameter declaration");
    let parameter_edits = service
        .rename(&path, parameter, "Minuend")
        .expect("parameter rename")
        .value;
    let label = source.find("Left := Left").expect("named label");
    let mut offsets = parameter_edits
        .iter()
        .map(|edit| edit.range.offset())
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    assert_eq!(
        offsets,
        [
            parameter,
            source.find("Left - Right").expect("body use"),
            label
        ],
        "{parameter_edits:?}"
    );

    let local = source
        .find("Left: integer := 10")
        .expect("local declaration");
    let local_edits = service
        .rename(&path, local, "Start")
        .expect("local rename")
        .value;
    assert_eq!(local_edits.len(), 2, "{local_edits:?}");
    assert!(local_edits.iter().all(|edit| edit.range.offset() != label));
}

#[test]
fn renaming_a_variant_field_updates_construction_and_pattern_labels_independently() {
    let temp = TempDirectory::new("references-named-variant-fields");
    let source = "program Named;\n\ntype Shape = enum\n  Rect(Width: integer; Height: integer);\nend enum;\n\nbegin\n  const Width: integer := 2;\n  const S: Shape := Shape.Rect(Height := 1, Width := Width);\n  case S of when Shape.Rect(Height := _, Width := const Captured): discard Captured; end case;\n  discard S is Shape.Rect(Width := 2, Height := _);\nend.\n";
    let path = temp.write("named.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let field = source.find("Width: integer;").expect("field declaration");
    let label = source.find("Width := Width").expect("named label");
    let mut offsets = service
        .rename(&path, field, "Span")
        .expect("field rename")
        .value
        .iter()
        .map(|edit| edit.range.offset())
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    assert_eq!(
        offsets,
        [
            field,
            label,
            source.find("Width := const").expect("case label"),
            source.find("Width := 2").expect("is label")
        ]
    );
    let binding_edits = service
        .rename(
            &path,
            source.find("Captured):").expect("binding"),
            "Payload",
        )
        .expect("binding rename")
        .value;
    assert_eq!(binding_edits.len(), 2, "{binding_edits:#?}");
    assert!(
        binding_edits
            .iter()
            .all(|edit| !offsets.contains(&edit.range.offset()))
    );

    assert!(matches!(
        service.rename(&path, field, "Height"),
        Err(RenameError::Conflict { .. })
    ));
}

#[test]
fn callable_type_label_rename_keeps_implementation_parameter_names() {
    let temp = TempDirectory::new("references-static-callable-labels");
    let source = "program T; function Double(N: integer): integer; begin return N * 2; end function; begin const F: function(Value: integer): integer := Double; discard F(Value := 3); end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let label = source.find("Value :=").expect("call label");
    let definitions = service
        .definitions(&path, label)
        .expect("definitions")
        .value;
    assert_eq!(definitions.len(), 1);
    assert_eq!(
        definitions[0].symbol.selection_span.offset(),
        source.find("Value: integer").expect("static parameter")
    );
    let edits = service
        .rename(&path, label, "Input")
        .expect("label rename")
        .value;
    let mut actual = edits
        .iter()
        .map(|edit| edit.range.offset())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            source.find("Value: integer").expect("static declaration"),
            label
        ]
    );
}

#[test]
fn inferred_callable_labels_follow_static_source_signatures_through_branches_and_arrays() {
    let temp = TempDirectory::new("references-inferred-callable-labels");
    let source = "program T; function Double(N: integer): integer; begin return N * 2; end function; begin const F: function(Value: integer): integer := Double; const G := Double; const Jobs := [G]; const H := if false then F else Double end if; discard G(N := 3); discard Jobs[0](N := 4); discard H(Value := 5); end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let edits = service
        .rename(
            &path,
            source.find("N: integer").expect("routine parameter"),
            "Input",
        )
        .expect("rename")
        .value;
    let mut offsets = edits
        .iter()
        .map(|edit| edit.range.offset())
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    assert_eq!(
        offsets,
        [
            source.find("N: integer").expect("parameter"),
            source.find("N * 2").expect("body"),
            source.find("N := 3").expect("inferred label"),
            source.find("N := 4").expect("indexed label")
        ]
    );
    let receiving_edits = service
        .rename(
            &path,
            source.find("Value: integer").expect("receiving parameter"),
            "Argument",
        )
        .expect("static rename")
        .value;
    assert_eq!(receiving_edits.len(), 2, "{receiving_edits:#?}");
    assert!(
        receiving_edits
            .iter()
            .any(|edit| edit.range.offset() == source.find("Value := 5").expect("branch label"))
    );
}

#[test]
fn renaming_a_record_field_updates_member_accesses() {
    let temp = TempDirectory::new("references-record-field");
    let source = "program Fields;\n\ntype Point = record\n  X: integer;\n  Y: integer;\nend record;\n\nbegin\n  const P: Point := Point( X := 1, Y := 2 );\n  const A: integer := P.X;\nend.\n";
    let path = temp.write("fields.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let field = source.find("X: integer;").expect("field declaration");
    let edits = service
        .rename(&path, field, "Left")
        .expect("field rename")
        .value;
    assert!(
        edits
            .iter()
            .any(|edit| edit.range.offset() == source.find("P.X").expect("access") + 2),
        "{edits:?}"
    );
}
