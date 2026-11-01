//! Complete-scope identity and saved/editor parity at the shared analysis boundary.

#![allow(clippy::expect_used, reason = "fixture failures carry local context")]

use crate::support::{TempDirectory, write_program_project};
use fpas_language_service::{
    CancellationToken, LanguageService, SourceQueryOperation, SourceSession,
};

#[test]
fn loose_editor_sources_are_frozen_before_any_semantic_query() {
    let temp = TempDirectory::new("loose-overlay-inventory");
    let saved = "program Loose; begin const Value := 1; end.";
    let dirty = saved.replace("1", "2");
    let source = temp.write("main.fpas", saved);
    let mut editor = LanguageService::load(&source);
    editor
        .documents_mut()
        .open_document(&source, 7, dirty.as_str())
        .expect("dirty buffer");
    let token = CancellationToken::new();
    let snapshot = {
        let mut session =
            SourceSession::load(&source, None, &editor.documents().open_snapshots(), &token)
                .expect("loose scope");
        assert_eq!(session.inventory().files().len(), 1);
        assert!(session.inventory().files()[0].is_overlay);
        assert_eq!(
            session.inventory().files()[0].bytes.as_ref(),
            dirty.as_bytes()
        );
        let result = session
            .query_position(
                &source,
                dirty.find("Value").expect("declaration"),
                SourceQueryOperation::Definition,
                10,
                &token,
            )
            .expect("declaration query");
        assert!(result.target.expect("target").target_ref.is_some());
        assert!(session.verify().expect("stable scope"));
        session.snapshot().clone()
    };
    std::fs::write(&source, &dirty).expect("save same bytes");
    let saved = SourceSession::load(&source, None, &[], &token).expect("saved scope");
    assert_eq!(&snapshot, saved.snapshot());
}

#[test]
fn shared_sources_keep_each_consumers_errors_and_import_allocation() {
    let temp = TempDirectory::new("source-scope-consumers");
    let workspace = temp.write(
        "suite.fpasworkspace",
        "[workspace]\nname='suite'\nmembers=['a/a.fpasprj','b/b.fpasprj']\n",
    );
    let a = temp.write(
        "a/a.fpasprj",
        "[project]\nname='a'\nkind='library'\n[sources]\ninclude=['*.fpas','../shared.fpas']\n",
    );
    temp.write(
        "b/b.fpasprj",
        "[project]\nname='b'\nkind='library'\n[sources]\ninclude=['*.fpas','../shared.fpas']\n",
    );
    let value =
        "unit Demo.Value; public function Seed(): integer; begin return 1; end function; end unit;";
    let selected = temp.write("a/value.fpas", value);
    let other = temp.write("b/value.fpas", "unit Demo.Value; public function Seed(): string; begin return 'wrong'; end function; end unit;");
    let source = "unit Demo.Shared; uses Demo.Value as V; public function Read(): integer; begin return V.Seed(); end function; end unit;";
    let shared = temp.write("shared.fpas", source);
    let cancel = CancellationToken::new();
    {
        let mut session = SourceSession::load(&workspace, None, &[], &cancel).expect("workspace");
        let result = session
            .query_position(
                &selected,
                value.find("Seed").expect("seed"),
                SourceQueryOperation::Definition,
                10,
                &cancel,
            )
            .expect("valid selected declaration with an invalid consumer");
        assert!(
            result.reasons.contains("analysis_errors"),
            "{:?}",
            result.reasons
        );
        assert!(result.diagnostics.iter().any(|d| {
            d.path.as_ref().is_some_and(|p| p.ends_with("shared.fpas")) && d.diagnostic.is_error()
        }));
        assert!(result.target.expect("declaration").target_ref.is_none());
        let error = session
            .query_position(
                &shared,
                source.find("Seed").expect("call"),
                SourceQueryOperation::Definition,
                10,
                &cancel,
            )
            .err()
            .expect("two direct owners are ambiguous");
        assert_eq!(error.code, "ambiguous_context");
    }
    // Equal exported signatures still do not make two physical import allocations identical.
    std::fs::write(&other, value.replace("return 1", "return 2"))
        .expect("valid second interpretation");
    {
        let mut session = SourceSession::load(&workspace, None, &[], &cancel).expect("workspace");
        let found = session
            .find_declarations("Read", true, 10, &cancel)
            .expect("checked discovery");
        assert!(found.reasons.contains("ambiguous_context"));
        assert!(found.symbols.iter().all(|s| s.target.target_ref.is_none()));
    }
    let mut session = SourceSession::load(&a, None, &[], &cancel).expect("explicit consumer");
    let result = session
        .query_position(
            &shared,
            source.find("Seed").expect("call"),
            SourceQueryOperation::Definition,
            10,
            &cancel,
        )
        .expect("one selected interpretation");
    assert!(result.reasons.is_empty(), "{:?}", result.reasons);
    assert!(
        result
            .target
            .expect("target")
            .target_ref
            .expect("checked reference")
            .file
            .ends_with("/a/value.fpas")
    );
}

#[test]
fn a_deleted_unrelated_open_buffer_does_not_join_or_break_a_checked_scope() {
    let temp = TempDirectory::new("source-scope-deleted-overlay");
    let (manifest, main, _) = write_program_project(&temp);
    let unrelated = temp.write("unrelated/old.fpas", "program Old; begin end.");
    let mut editor = LanguageService::load(&manifest);
    editor
        .documents_mut()
        .open_document(&unrelated, 1, "program Old; begin discard 'dirty'; end.")
        .expect("unrelated buffer");
    std::fs::remove_file(&unrelated).expect("external deletion");
    let cancel = CancellationToken::new();
    let mut scope = SourceSession::load(
        &manifest,
        None,
        &editor.documents().open_snapshots(),
        &cancel,
    )
    .expect("selected scope");
    let source = std::fs::read_to_string(&main).expect("main");
    let result = scope
        .query_position(
            &main,
            source.find("Answer").expect("call"),
            SourceQueryOperation::Inspect,
            100,
            &cancel,
        )
        .expect("inspection");
    assert!(result.reasons.is_empty(), "{:?}", result.reasons);
    assert!(scope.verify().expect("verified physical scope"));
    assert!(
        scope
            .snapshot()
            .inventory
            .files
            .iter()
            .all(|f| !f.path.ends_with("old.fpas"))
    );
    assert!(
        scope
            .query_position(&unrelated, 8, SourceQueryOperation::Inspect, 100, &cancel)
            .is_err(),
        "a deleted buffer cannot authorize a physical checked input"
    );
}

#[test]
fn scope_is_operation_independent_and_unedited_inputs_invalidate_it() {
    let temp = TempDirectory::new("source-scope");
    let (manifest, main, unit) = write_program_project(&temp);
    let cancel = CancellationToken::new();
    let mut scope = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let before = scope.snapshot().clone();
    let source = std::fs::read_to_string(&main).expect("main");
    let offset = source.find("Answer").expect("call");
    for operation in [
        SourceQueryOperation::References,
        SourceQueryOperation::Definition,
        SourceQueryOperation::Type,
    ] {
        let result = scope
            .query_position(&main, offset, operation, 1, &cancel)
            .expect("query");
        assert!(result.reasons.is_empty(), "{:?}", result.reasons);
        assert!(scope.verify().expect("scope verification"));
        assert_eq!(scope.snapshot(), &before);
        let reference = result
            .target
            .expect("declaration")
            .target_ref
            .expect("checked reference");
        let reused = scope
            .query_reference(&reference, SourceQueryOperation::Definition, 10, &cancel)
            .expect("reuse");
        assert_eq!(
            reused.target.expect("same declaration").target_ref.as_ref(),
            Some(&reference)
        );
    }
    let original = std::fs::read_to_string(&unit).expect("unit");
    std::fs::write(&unit, format!("{original}\n// changed dependency\n"))
        .expect("change dependency");
    assert!(!scope.verify().expect("changed scope"));
    drop(scope);
    let changed = SourceSession::load(&manifest, None, &[], &cancel).expect("new scope");
    assert_ne!(before.id, changed.snapshot().id);
}

#[test]
fn dirty_multifile_analysis_matches_the_same_saved_bytes_and_not_editor_versions() {
    let temp = TempDirectory::new("source-scope-overlays");
    let (manifest, main, unit) = write_program_project(&temp);
    let cancel = CancellationToken::new();
    let clean = SourceSession::load(&manifest, None, &[], &cancel).expect("clean scope");
    let clean_id = clean.snapshot().id.clone();
    drop(clean);
    let main_text = std::fs::read_to_string(&main)
        .expect("main")
        .replace("integer", "string");
    let unit_text = std::fs::read_to_string(&unit)
        .expect("unit")
        .replace("integer", "string")
        .replace("42", "'dirty'");
    let mut editor = LanguageService::load(&manifest);
    editor
        .documents_mut()
        .open_document(&main, 17, main_text.as_str())
        .expect("dirty main");
    editor
        .documents_mut()
        .open_document(&unit, 6, unit_text.as_str())
        .expect("dirty dependency");
    let overlays = editor.documents().open_snapshots();
    let mut dirty = SourceSession::load(&manifest, None, &overlays, &cancel).expect("dirty scope");
    let result = dirty
        .service()
        .source_query(
            &main,
            main_text.find("Value").expect("binding"),
            SourceQueryOperation::Type,
            10,
            &cancel,
        )
        .expect("dirty type");
    assert_eq!(
        result.expression_type.expect("checked type").type_name,
        "string"
    );
    assert!(result.reasons.is_empty(), "{:?}", result.reasons);
    assert!(dirty.verify().expect("overlay verification"));
    let dirty_snapshot = dirty.snapshot().clone();
    assert_ne!(dirty_snapshot.id, clean_id);
    drop(dirty);
    std::fs::write(&main, main_text).expect("save main");
    std::fs::write(&unit, unit_text).expect("save unit");
    let saved = SourceSession::load(&manifest, None, &[], &cancel).expect("saved scope");
    assert_eq!(saved.snapshot(), &dirty_snapshot);
}

#[test]
fn discovery_membership_changes_invalidate_an_existing_session() {
    let temp = TempDirectory::new("source-scope-membership");
    let (manifest, _, _) = write_program_project(&temp);
    let cancel = CancellationToken::new();
    let scope = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let before = scope.snapshot().id.clone();
    temp.write("src/extra.fpas", "unit Demo.Extra; end unit;\n");
    assert!(!scope.verify().expect("membership verification"));
    drop(scope);
    let changed = SourceSession::load(&manifest, None, &[], &cancel).expect("expanded scope");
    assert_ne!(before, changed.snapshot().id);
}

#[cfg(any(unix, windows))]
#[test]
fn a_reference_reconstructs_the_original_lexical_path_bindings() {
    let temp = TempDirectory::new("source-scope-alias");
    temp.write(
        "real/app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let text = "program Main; const Value: integer := 1; begin discard Value; end.";
    temp.write("real/main.fpas", text);
    let alias = temp.join("alias");
    #[cfg(windows)]
    {
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&alias)
            .arg(temp.join("real"))
            .output()
            .expect("create junction");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(temp.join("real"), &alias).expect("create alias");
    let cancel = CancellationToken::new();
    let mut original =
        SourceSession::load(&alias.join("app.fpasprj"), None, &[], &cancel).expect("alias scope");
    let selected = original
        .query_position(
            &alias.join("main.fpas"),
            text.find("Value").expect("declaration"),
            SourceQueryOperation::Definition,
            10,
            &cancel,
        )
        .expect("alias target");
    let reference = selected
        .target
        .expect("target")
        .target_ref
        .expect("reference");
    drop(original);
    let mut reused = SourceSession::load(
        std::path::Path::new(&reference.context.input),
        None,
        &[],
        &cancel,
    )
    .expect("reconstructed scope");
    let result = reused.query_reference(&reference, SourceQueryOperation::Definition, 10, &cancel);
    #[cfg(windows)]
    std::fs::remove_dir(&alias).expect("remove junction");
    #[cfg(unix)]
    std::fs::remove_file(&alias).expect("remove alias");
    assert!(result.is_ok(), "{:?}", result.err());
}
