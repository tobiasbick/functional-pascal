//! Editor diagnostics for invalid collection callbacks.

use std::path::Path;

use crate::LanguageService;

#[test]
fn invalid_collection_callbacks_reach_editor_diagnostics() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("language-service crate is inside the repository");
    for (source, expected) in [
        (
            "tests/stdlib/str/filter_wrong_callback_compile_error.fpas",
            "callback return type",
        ),
        (
            "tests/stdlib/dict/reduce_wrong_callback_arity_compile_error.fpas",
            "callback must take",
        ),
    ] {
        let path = repository.join(source);
        let mut service =
            LanguageService::load_with_standard_library(&path, &repository.join("lib"))
                .expect("standard library loads");
        let analysis = service
            .analyze_document_diagnostics(&path)
            .expect("source analysis succeeds");
        assert!(
            analysis.failure().is_none(),
            "{source}: project analysis failed"
        );
        assert!(
            analysis
                .document()
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{source}: expected {expected} in editor diagnostics"
        );
    }
}
