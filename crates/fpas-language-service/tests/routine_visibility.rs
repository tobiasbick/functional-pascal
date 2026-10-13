//! Editor resolution uses the same scope-wide signatures and lexical shadowing as the compiler.

mod support;

use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn forward_nested_routines_shadow_outer_names_in_navigation_and_completion() {
    let temp = TempDirectory::new("forward-routine");
    let source = "program T;
function Read(): integer; begin return 1; end function;
function Outer(): integer;
  function Use(): integer; begin return Read(); end function;
  function Read(): integer; begin return Later; end function;
begin return Use(); end function;
const Later: integer := 2;
begin discard Outer(); end.";
    let path = temp.write("forward.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let use_offset = source.find("return Read()").unwrap() + "return ".len();
    let selected = service.definitions(&path, use_offset).unwrap().value;
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].symbol.selection_span.offset(),
        source.rfind("function Read").unwrap() + "function ".len()
    );
    let completions = service.completions(&path, use_offset + 2).unwrap().value;
    let reads: Vec<_> = completions
        .iter()
        .filter(|candidate| candidate.label == "Read")
        .collect();
    assert_eq!(reads.len(), 1);
    assert!(
        reads[0].owner.as_deref().unwrap().contains("Outer"),
        "{reads:?}"
    );
    let later_use = source.find("return Later").unwrap() + "return ".len();
    let selected = service.definitions(&path, later_use).unwrap().value;
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].symbol.selection_span.offset(),
        source.find("const Later").unwrap() + "const ".len()
    );
    let references = service.references(&path, use_offset, true).unwrap().value;
    assert!(
        references
            .iter()
            .any(|reference| reference.span.offset() == use_offset)
    );
    assert!(references.iter().all(|reference| reference.span.offset()
        != source.find("function Read").unwrap() + "function ".len()));
}

#[test]
fn anonymous_body_navigation_retains_the_compilers_later_binding_identity() {
    let temp = TempDirectory::new("forward-anonymous-value");
    let source = "program T;
const Stored: function(): integer := function(): integer begin return Later; end function;
const Later: integer := 7;
begin discard Stored(); end.";
    let path = temp.write("anonymous.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let offset = source.find("return Later").unwrap() + "return ".len();
    let selected = service.definitions(&path, offset).unwrap().value;
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].symbol.selection_span.offset(),
        source.find("const Later").unwrap() + "const ".len()
    );
}
