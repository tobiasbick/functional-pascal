//! Clause result bindings resolve independently and never leak into the body.
#![allow(
    clippy::expect_used,
    reason = "navigation fixtures use explicit source offsets"
)]

mod support;

use fpas_language_service::{LanguageService, SymbolKind, WorkspaceContext};
use support::TempDirectory;

#[test]
fn postcondition_result_names_support_hover_definition_and_clause_local_scope() {
    let temp = TempDirectory::new("contracts");
    let source = "program T; const Value: integer := 1;
        function F(X: integer): integer;
        ensures (Value) Value = X;
        ensures (Value) Value > 0;
        ensures Value = 1;
        begin return Value; end function;
        begin const N: integer := F(1); end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (read, declaration) in [
        ("Value = X", source.find("(Value)").expect("first name") + 1),
        (
            "Value > 0",
            source.rfind("(Value)").expect("second name") + 1,
        ),
        ("Value = 1", source.find("Value:").expect("constant")),
        (
            "Value; end function",
            source.find("Value:").expect("constant"),
        ),
    ] {
        let offset = source.find(read).expect("use");
        let definitions = service
            .definitions(&path, offset)
            .expect("definition query")
            .value;
        assert_eq!(definitions.len(), 1, "{read}: {definitions:?}");
        assert_eq!(
            definitions[0].symbol.selection_span.offset(),
            declaration,
            "{read}"
        );
        assert_eq!(definitions[0].symbol.kind, SymbolKind::Constant);
        let hover = service
            .hover(&path, offset)
            .expect("hover query")
            .value
            .expect("hover");
        assert!(hover.contents.contains("const Value: integer"), "{hover:?}");
        if read == "Value = X" || read == "Value > 0" {
            let edits = service
                .rename(&path, offset, "CheckedValue")
                .expect("clause-local rename")
                .value;
            assert_eq!(edits.len(), 2, "{edits:?}");
            assert!(
                edits
                    .iter()
                    .all(|edit| [declaration, offset].contains(&edit.range.offset())),
                "{edits:?}"
            );
        }
    }
}
