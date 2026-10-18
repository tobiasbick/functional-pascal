//! Resolved members through non-lexical receivers retain source declaration identity.

#![expect(
    clippy::expect_used,
    reason = "Source fixture assertions name their observable selection."
)]
mod support;
use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn fields_after_calls_and_indices_never_resolve_to_a_same_named_lexical_value() {
    let temp = TempDirectory::new("checked-members");
    let source = "program T;\n\
type Box of T = record\n  Value: T;\nend record;\n\
function Wrap<T>(Item: T): Box of T;\nbegin\n  return Box(Value := Item);\nend function;\n\
begin\n  const Value := 123;\n  const Items := [Wrap('a')];\n\
  discard Wrap('b').Value;\n  discard Items[0].Value;\n  discard Value;\nend.\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let declaration = source.find("Value: T").expect("field declaration");
    for needle in [".Value;\n  discard Items", ".Value;\n  discard Value"] {
        let selected = source.find(needle).expect("member") + 1;
        let result = service
            .definitions(&path, selected)
            .expect("definition")
            .value;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.selection_span.offset(), declaration);
    }
    let references = service
        .references(&path, declaration, false)
        .expect("references")
        .value;
    assert_eq!(references.len(), 3, "{references:?}");
    assert!(
        !references
            .iter()
            .any(|r| r.span.offset() == source.find("discard Value;").expect("lexical") + 8)
    );
}

#[test]
fn imported_standard_source_errors_reduce_coverage_with_their_own_provenance() {
    use fpas_language_service::{CancellationToken, SourceQueryOperation};
    for body in ["return 'wrong';", "return 1 + ;"] {
        let _inputs = fpas_project::source_inputs::SourceInputs::begin(|| false).expect("inputs");
        let temp = TempDirectory::new("query-import-errors");
        temp.write(
            "lib/stdlib.fpasprj",
            "[project]\nname='Std'\nkind='library'\n[sources]\ninclude=['*.fpas']\n",
        );
        temp.write("lib/example.fpas", &format!("unit Std.Example; public function Read(): integer; begin {body} end function; end unit;"));
        let source = "program T; uses Std.Example as E; begin discard E.Read(); end.";
        let path = temp.write("main.fpas", source);
        let mut service =
            LanguageService::load_with_standard_library(&path, &temp.path().join("lib"))
                .expect("library");
        let result = service
            .source_query(
                &path,
                source.find("Read()").expect("call"),
                SourceQueryOperation::Type,
                1000,
                &CancellationToken::new(),
            )
            .expect("query");
        assert!(
            result.reasons.contains("analysis_errors"),
            "{:?}",
            result.diagnostics
        );
        assert!(result.diagnostics.iter().any(|d| {
            d.path.as_ref().is_some_and(|p| p.ends_with("example.fpas")) && d.diagnostic.is_error()
        }));
    }
}
