//! Alias renames and the no-shadowing rule.

use fpas_language_service::RenameError;

use super::{apply_edits, fixture};

#[test]
fn alias_rename_edits_only_local_binding_and_qualifiers() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  const Local: M.Point := M.MakePoint();\n  discard m.Answer(1);\nend.\n";
    let mut f = fixture(source);
    let edits = f
        .service
        .rename(
            &f.main,
            source.find("m.Answer").expect("alias use"),
            "Maths",
        )
        .expect("rename alias")
        .value;
    assert_eq!(edits.len(), 4);
    assert!(edits.iter().all(|edit| edit.path == f.main));
    let edited = apply_edits(source, &edits, &f.main);
    assert!(edited.contains("Demo.Math as Maths"));
    assert!(edited.contains("Maths.Point"));
    assert!(edited.contains("Maths.MakePoint()"));
    f.service
        .documents_mut()
        .open_document(&f.main, 1, edited)
        .expect("apply alias rename");
    assert!(
        f.service
            .analyze_document(&f.main)
            .expect("renamed analysis")
            .diagnostics()
            .is_empty()
    );
}

#[test]
fn local_parameter_and_generic_names_cannot_be_renamed_to_alias_even_if_unused() {
    let source = "program App;\nuses Demo.Math as M;\nfunction Identity<T>(Value: T): T;\nbegin\n  return Value;\nend function;\nbegin\n  const Local: integer := 1;\nend.\n";
    let mut f = fixture(source);
    for offset in [
        source.find("Local:").expect("local"),
        source.find("Value:").expect("parameter"),
        source.find("<T>").expect("generic") + 1,
    ] {
        assert!(matches!(
            f.service.rename(&f.main, offset, "m"),
            Err(RenameError::Conflict { .. })
        ));
    }
}

#[test]
fn alias_rename_rejects_local_bindings_and_namespace_roots_case_insensitively() {
    let source = "program App;\nuses Demo.Math as M;\nfunction GetValue(Value: integer): integer;\nbegin return Value; end function;\nbegin\n  const Later: integer := 1;\nend.\n";
    let mut f = fixture(source);
    let offset = source.find("as M").expect("alias declaration") + 3;
    for name in [
        "value", "LATER", "GetValue", "Demo", "STD", "App", "integer",
    ] {
        let result = f.service.rename(&f.main, offset, name);
        assert!(
            matches!(
                result,
                Err(RenameError::Conflict { .. }) | Err(RenameError::InvalidIdentifier { .. })
            ),
            "{name}: {result:?}"
        );
    }
}

#[test]
fn alias_can_be_named_as_and_record_fields_do_not_shadow_aliases() {
    let source = "program App;\nuses Demo.Math as M;\ntype Holder = record\n  M: integer;\nend record;\nbegin\n  discard M.Answer(1);\nend.\n";
    let mut f = fixture(source);
    let edits = f
        .service
        .rename(&f.main, source.find("M.Answer").expect("alias"), "As")
        .expect("ordinary alias name")
        .value;
    assert_eq!(edits.len(), 2);
    assert!(apply_edits(source, &edits, &f.main).contains("as As"));
}
