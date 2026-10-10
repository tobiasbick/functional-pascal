//! Symbol classification for ordinary source names.
//!
//! **Documentation:** `docs/pascal/getting-started/keywords.md`

use super::*;

#[test]
fn ordinary_names_receive_their_declared_symbol_kinds() {
    let temp = TempDirectory::new("ordinary-names");
    let source = "unit Ordinary.Names;
      public type Event = record public Nil: integer; end record;
      public function Assigned(Read: integer; Write: integer): integer;
      begin return Read + Write; end function;
      end unit;";
    let path = temp.write("names.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let tokens = service
        .semantic_tokens(&path)
        .expect("semantic tokens")
        .value;

    for (name, kind) in [
        ("Event", SemanticTokenKind::Type),
        ("Nil", SemanticTokenKind::Field),
        ("Assigned", SemanticTokenKind::Function),
        ("Read", SemanticTokenKind::Parameter),
        ("Write", SemanticTokenKind::Parameter),
    ] {
        let matching = tokens
            .iter()
            .filter(|token| &source[token.span.offset()..token.span.end()] == name)
            .collect::<Vec<_>>();
        assert!(!matching.is_empty(), "missing {name}");
        assert!(
            matching.iter().all(|token| token.kind == kind),
            "{matching:#?}"
        );
    }
}
