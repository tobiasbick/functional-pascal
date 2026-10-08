//! Completion and auto-import namespace reuse.

use fpas_language_service::{CompletionKind, CompletionSource, SymbolKind};

use super::fixture;

#[test]
fn alias_members_are_public_and_keep_canonical_metadata() {
    let source = "program App;\nuses dEmO.mAtH as M;\nbegin\n  discard m.Ans;\nend.\n";
    let mut f = fixture(source);
    let candidates = f
        .service
        .completions(&f.main, source.find("m.Ans").expect("member") + 5)
        .expect("completion")
        .value;
    let answer = candidates
        .iter()
        .find(|c| c.label == "Answer")
        .expect("public callable");
    assert_eq!(answer.qualified_name, "Demo.Math.Answer");
    assert_eq!(answer.owner.as_deref(), Some("Demo.Math"));
    assert_eq!(answer.insert_text, "Answer");
    assert!(answer.additional_edit.is_none());

    let all_source = source.replace("m.Ans", "M.");
    f.service
        .documents_mut()
        .open_document(&f.main, 1, all_source.clone())
        .expect("open buffer");
    let all = f
        .service
        .completions(&f.main, all_source.find("M.;").expect("namespace") + 2)
        .expect("all members")
        .value;
    assert!(all.iter().any(|c| c.label == "Point"));
    assert!(
        !all.iter()
            .any(|c| c.label == "Secret" || c.label == "Hidden" || c.label == "Trim")
    );
}

#[test]
fn existing_alias_auto_import_qualifies_without_changing_uses() {
    let source = "program App;\nuses dEmO.mAtH // preserve this comment\n  as Maths;\nbegin\n  discard Ans;\nend.\n";
    let mut f = fixture(source);
    let candidates = f
        .service
        .completions(
            &f.main,
            source.find("discard Ans").expect("prefix") + "discard Ans".len(),
        )
        .expect("completion")
        .value;
    let answer = candidates
        .iter()
        .find(|c| c.qualified_name == "Demo.Math.Answer")
        .expect("alias reuse");
    assert_eq!(answer.source, CompletionSource::AutoImport);
    assert_eq!(answer.label, "Maths.Answer");
    assert_eq!(answer.insert_text, "Maths.Answer");
    assert_eq!(answer.filter_text, "Answer");
    assert!(answer.additional_edit.is_none());
    let mut edited = source.to_owned();
    edited.replace_range(
        answer.replacement_span.offset()..answer.replacement_span.end(),
        "Maths.Answer(1)",
    );
    f.service
        .documents_mut()
        .open_document(&f.main, 1, edited)
        .expect("apply completion");
    assert!(
        f.service
            .analyze_document(&f.main)
            .expect("valid completed program")
            .diagnostics()
            .is_empty()
    );
}

#[test]
fn plain_import_is_reused_case_insensitively_and_alias_is_offered_as_namespace() {
    for (import, expected, kind) in [
        ("dEmO.mAtH", "Answer", SymbolKind::Function),
        ("Demo.Math as Maths", "Maths", SymbolKind::ImportAlias),
    ] {
        let source =
            format!("program App;\nuses {import};\nbegin\n  discard Ans;\n  discard Mat;\nend.\n");
        let mut f = fixture(&source);
        let prefix = if kind == SymbolKind::ImportAlias {
            "Mat;"
        } else {
            "Ans;"
        };
        let candidates = f
            .service
            .completions(&f.main, source.find(prefix).expect("prefix") + 3)
            .expect("completion")
            .value;
        let candidate = candidates
            .iter()
            .find(|c| c.label == expected)
            .expect("reused import");
        assert_eq!(candidate.source, CompletionSource::Declaration);
        assert_eq!(candidate.kind, CompletionKind::Symbol(kind));
        assert!(candidate.additional_edit.is_none());
    }
}

#[test]
fn adding_another_unit_preserves_existing_aliases() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  discard Extra;\nend.\n";
    let mut f = fixture(source);
    f.temp.write("src/extra.fpas", "unit Demo.Extra;\npublic function ExtraValue(): integer;\nbegin return 2; end function;\nend unit;\n");
    f.service = fpas_language_service::LanguageService::load(&f.temp.join("demo.fpasprj"));
    let candidates = f
        .service
        .completions(&f.main, source.find("Extra;").expect("prefix") + 5)
        .expect("completion")
        .value;
    let candidate = candidates
        .iter()
        .find(|c| c.label == "ExtraValue")
        .expect("new import");
    let edit = candidate.additional_edit.as_ref().expect("uses edit");
    assert!(edit.new_text.contains("Demo.Math as M"));
    assert_eq!(edit.new_text.matches("Demo.Math").count(), 1);
    assert_eq!(edit.new_text.matches("Demo.Extra").count(), 1);
}

#[test]
fn hidden_original_unit_path_and_short_symbols_are_not_declaration_completions() {
    for receiver in ["Demo.Math.", ""] {
        let source =
            format!("program App;\nuses Demo.Math as M;\nbegin\n  discard {receiver}Ans;\nend.\n");
        let mut f = fixture(&source);
        let candidates = f
            .service
            .completions(&f.main, source.rfind("Ans;").expect("prefix") + 3)
            .expect("completion")
            .value;
        assert!(
            !candidates
                .iter()
                .any(|c| c.label == "Answer" && c.source == CompletionSource::Declaration)
        );
    }
}

#[test]
fn aliased_record_values_expose_public_members() {
    let source = "program App;\nuses Demo.Math as M;\nbegin\n  const Local: M.Point := M.MakePoint();\n  discard Local.X;\n  discard M.Origin.X;\nend.\n";
    let mut f = fixture(source);
    for name in ["Local.X", "M.Origin.X"] {
        let candidates = f
            .service
            .completions(&f.main, source.find(name).expect("member") + name.len())
            .expect("record completion")
            .value;
        assert!(candidates.iter().any(|c| c.label == "X"));
        assert!(!candidates.iter().any(|c| c.label == "Hidden"));
    }
}

#[test]
fn existing_alias_takes_precedence_over_unimported_equal_short_names() {
    for imports in ["Demo.Math as M", "Demo.Math as M, Demo.Other as O"] {
        let source = format!("program App;\nuses {imports};\nbegin\n  discard Ans;\nend.\n");
        let mut f = fixture(&source);
        f.temp.write("src/other.fpas", "unit Demo.Other;\npublic function Answer(Value: integer): integer;\nbegin return Value; end function;\nend unit;\n");
        f.service = fpas_language_service::LanguageService::load(&f.temp.join("demo.fpasprj"));
        let candidates = f
            .service
            .completions(&f.main, source.find("Ans;").expect("prefix") + 3)
            .expect("alias precedence")
            .value;
        let answers = candidates
            .iter()
            .filter(|c| c.filter_text == "Answer")
            .collect::<Vec<_>>();
        assert!(answers.iter().any(|c| c.insert_text == "M.Answer"));
        assert!(answers.iter().all(|c| c.additional_edit.is_none()));
        assert_eq!(answers.len(), if imports.contains("Other") { 2 } else { 1 });
    }
}
