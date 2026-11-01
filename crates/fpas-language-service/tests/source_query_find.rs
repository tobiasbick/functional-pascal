//! Checked discovery, bounded ranking, exact selection and reference reuse through real projects.
#![allow(
    clippy::expect_used,
    reason = "fixtures identify each required semantic observation"
)]

use crate::support::{TempDirectory, write_program_project};
use fpas_language_service::{
    CancellationToken, DeclarationOrigin, SourceQueryOperation, SourceSession,
};

#[test]
fn names_rank_exactly_and_clipping_never_turns_ambiguity_into_selection() {
    let temp = TempDirectory::new("checked-find-ranking");
    let (manifest, _, _) = write_program_project(&temp);
    let declarations = (0..130)
        .map(|n| format!("const Answer{n:03}: integer := {n};\n"))
        .collect::<String>();
    temp.write("src/extra.fpas", &format!("unit A.Extra;\n{declarations}\npublic function Answer(): integer; begin return 1; end function; end unit;"));
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let found = session
        .find_declarations("answer", false, 1, &cancel)
        .expect("find");
    assert_eq!(found.known_total, Some(132));
    assert_eq!(found.symbols.len(), 1);
    assert_eq!(found.symbols[0].target.qualified_name, "A.Extra.Answer");
    assert!(found.reasons.contains("result_limit"));
    assert!(found.symbols[0].target.target_ref.is_some());
    let selected = session
        .query_symbol("aNsWeR", SourceQueryOperation::Definition, 1, &cancel)
        .expect("selection");
    assert_eq!(selected.selection_error, Some("ambiguous_target"));
    assert_eq!(selected.candidate_total, Some(2));
    assert_eq!(selected.candidates.len(), 1);
    assert!(selected.target.is_none());
    let exact = session
        .find_declarations("DEMO.Math.Answer", false, 1, &cancel)
        .expect("qualified find");
    assert_eq!(exact.symbols[0].target.qualified_name, "Demo.Math.Answer");
    let reference = exact.symbols[0]
        .target
        .target_ref
        .as_ref()
        .expect("portable reference");
    let reused = session
        .query_reference(reference, SourceQueryOperation::References, 10, &cancel)
        .expect("reuse find reference");
    assert_eq!(reused.matches.len(), 1);
    let ty = session
        .query_symbol("Demo.Math.Answer", SourceQueryOperation::Type, 10, &cancel)
        .expect("routine value type");
    assert_eq!(ty.expression_type.expect("callable type").kind, "callable");
    let absent = session
        .query_symbol(
            "Demo.Math.Ans",
            SourceQueryOperation::Definition,
            10,
            &cancel,
        )
        .expect("exact only");
    assert_eq!(absent.selection_error, Some("no_target"));
}

#[test]
fn dependencies_expose_imported_exports_and_owned_context_exposes_private_declarations() {
    let temp = TempDirectory::new("checked-find-export");
    let library = temp.write("lib/lib.fpasprj", "[project]\nname='lib'\nkind='library'\n[sources]\ninclude=['*.fpas']\n[exports]\nunits=['Lib.Api']\n");
    temp.write("lib/api.fpas", "unit Lib.Api; public function PublicValue(): integer; begin return 1; end function; function PrivateValue(): integer; begin return 2; end function; end unit;");
    temp.write(
        "lib/hidden.fpas",
        "unit Lib.Hidden; public const HiddenValue: integer := 3; end unit;",
    );
    let manifest = temp.write("app/app.fpasprj", "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n[dependencies]\nprojects=['../lib/lib.fpasprj']\n");
    temp.write(
        "app/main.fpas",
        "program App; uses Lib.Api as Api; begin const LocalValue := Api.PublicValue(); end.",
    );
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("consumer");
    let found = session
        .find_declarations("Value", false, 100, &cancel)
        .expect("exports");
    assert!(found.reasons.is_empty(), "{:?}", found.reasons);
    let names = found
        .symbols
        .iter()
        .filter(|s| s.target.snapshot.is_some())
        .map(|s| s.target.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["LocalValue", "PublicValue"]);
    let public = found
        .symbols
        .iter()
        .find(|s| s.target.name == "PublicValue")
        .expect("export");
    assert_eq!(public.origin, DeclarationOrigin::Dependency);
    let reference = public.target.target_ref.as_ref().expect("export reference");
    let reused = session
        .query_reference(reference, SourceQueryOperation::Definition, 10, &cancel)
        .expect("reuse dependency reference");
    assert_eq!(reused.matches.len(), 1);
    let alias = session
        .query_symbol("Api", SourceQueryOperation::Definition, 10, &cancel)
        .expect("alias excluded");
    assert_eq!(alias.selection_error, Some("no_target"));
    drop(session);
    let mut own = SourceSession::load(&library, None, &[], &cancel).expect("library");
    let found = own
        .find_declarations("PrivateValue", true, 10, &cancel)
        .expect("owned private");
    assert_eq!(found.known_total, Some(1));
    assert_eq!(found.symbols[0].origin, DeclarationOrigin::Owned);
}

#[test]
fn broken_unedited_sources_never_prove_unique_or_missing_names() {
    let temp = TempDirectory::new("checked-find-errors");
    let (manifest, _, _) = write_program_project(&temp);
    temp.write("src/broken.fpas", "unit Broken; public function Wrong(): integer; begin return 'wrong'; end function; end unit;");
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    for name in ["Answer", "DoesNotExist"] {
        let result = session
            .query_symbol(name, SourceQueryOperation::Definition, 10, &cancel)
            .expect("partial selection");
        assert!(result.target.is_none());
        assert!(result.selection_error.is_none());
        assert!(result.reasons.contains("analysis_errors"));
        assert!(result.candidate_total.is_none());
        assert!(
            result
                .candidates
                .iter()
                .all(|c| c.target.target_ref.is_none())
        );
        assert!(result.diagnostics.iter().any(|d| {
            d.path.as_ref().is_some_and(|p| p.ends_with("broken.fpas")) && d.diagnostic.is_error()
        }));
    }
}

#[test]
fn native_operations_have_identity_without_inventing_a_declaration_span() {
    let temp = TempDirectory::new("checked-find-native");
    let (manifest, _, _) = write_program_project(&temp);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let found = session
        .find_declarations("string.TrimLeft", true, 10, &cancel)
        .expect("native");
    assert_eq!(found.known_total, Some(1));
    assert_eq!(found.symbols[0].origin, DeclarationOrigin::Builtin);
    assert!(found.symbols[0].target.snapshot.is_none());
    assert_eq!(
        found.symbols[0].target.reference_unavailable,
        Some("no_source")
    );
}

#[test]
fn imported_intrinsics_without_api_sources_still_have_checked_identity_and_type() {
    let temp = TempDirectory::new("checked-find-intrinsic-without-source");
    let (manifest, main, _) = write_program_project(&temp);
    std::fs::write(main, "program Main; uses Std.Io.Fs as Fs; begin end.")
        .expect("intrinsic import");
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let found = session
        .find_declarations("Std.Io.Fs.ReadText", true, 10, &cancel)
        .expect("intrinsic");
    assert_eq!(found.known_total, Some(1));
    assert_eq!(
        found.symbols[0].target.kind,
        Some(fpas_language_service::SymbolKind::Function)
    );
    assert_eq!(
        found.symbols[0].target.reference_unavailable,
        Some("no_source")
    );
    let selected = session
        .query_symbol(
            "Std.Io.Fs.ReadText",
            SourceQueryOperation::Type,
            10,
            &cancel,
        )
        .expect("value type");
    assert_eq!(
        selected.declaration_type.expect("compiler type").kind,
        "callable"
    );
    assert!(selected.expression_type.is_none());
}

#[test]
fn imported_intrinsic_api_references_use_the_registered_callable_type() {
    let temp = TempDirectory::new("checked-find-intrinsic-api");
    let (manifest, main, _) = write_program_project(&temp);
    std::fs::write(main, "program Main; uses Std.Io.Fs as Fs; begin end.")
        .expect("intrinsic import");
    let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib");
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, Some(&library), &[], &cancel).expect("scope");
    let found = session
        .find_declarations("Std.Io.Fs.ReadText", true, 10, &cancel)
        .expect("find API");
    assert!(found.reasons.is_empty(), "{:?}", found.reasons);
    assert_eq!(found.known_total, Some(1));
    let reference = found.symbols[0]
        .target
        .target_ref
        .as_ref()
        .expect("API source reference");
    for operation in [
        SourceQueryOperation::Definition,
        SourceQueryOperation::References,
        SourceQueryOperation::Type,
    ] {
        let position = session
            .query_position(
                std::path::Path::new(&reference.file),
                reference.start,
                operation,
                10,
                &cancel,
            )
            .expect("API declaration position");
        assert_eq!(
            position.target.as_ref().and_then(|t| t.target_ref.as_ref()),
            Some(reference),
            "{operation:?}: {:?}; {:?}",
            position.reasons,
            position.diagnostics
        );
        let reused = session
            .query_reference(reference, operation, 10, &cancel)
            .expect("API reference reuse");
        assert!(reused.reasons.is_empty(), "{:?}", reused.reasons);
        if operation == SourceQueryOperation::Type {
            assert_eq!(
                reused.expression_type.expect("registered callable").kind,
                "callable"
            );
        }
    }
}

#[test]
fn primitive_and_container_types_are_searchable_without_fabricated_source() {
    let temp = TempDirectory::new("checked-find-builtin-types");
    let (manifest, _, _) = write_program_project(&temp);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    for name in ["integer", "Result", "array"] {
        let found = session
            .find_declarations(name, true, 10, &cancel)
            .expect("builtin find");
        assert_eq!(found.known_total, Some(1));
        assert!(found.reasons.is_empty());
        let target = &found.symbols[0].target;
        assert_eq!(target.kind, Some(fpas_language_service::SymbolKind::Type));
        assert!(target.snapshot.is_none() && target.target_ref.is_none());
        assert_eq!(target.reference_unavailable, Some("no_source"));
        let selected = session
            .query_symbol(name, SourceQueryOperation::Inspect, 10, &cancel)
            .expect("builtin inspection");
        assert!(selected.target.is_some());
        assert!(selected.inspection.is_some());
    }
}
