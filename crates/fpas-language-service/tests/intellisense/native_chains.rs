//! Completion regressions for recovered native chains with recursive receiver types.
//! Reference: `docs/pascal/tools/editor-integration.md`.

use super::*;
use fpas_sema::Ty;
use std::collections::BTreeSet;

#[test]
fn index_search_chains_offer_option_operations_and_option_signature_results() {
    for expression in [
        "[1].IndexOf(1)",
        "[1].FindIndex(function(X: integer): boolean begin return true; end function)",
        "'abc'.IndexOf('b')",
        "'abc'.LastIndexOf('b')",
    ] {
        let temp = TempDirectory::new("native-search-option-chain");
        let source =
            format!("program Demo; begin const Index := {expression}; discard Index.\nend.");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find("Index.\n").expect("chain") + "Index.".len();
        let candidates = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        let unwrap = candidates
            .iter()
            .find(|candidate| candidate.qualified_name == "Option.Unwrap")
            .expect("Option operations on index-search result");
        assert!(
            unwrap.detail.ends_with(": integer"),
            "{expression}: {unwrap:#?}"
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.qualified_name == "Option.OrElse"),
            "{expression}: {candidates:#?}"
        );
    }
    for (receiver, method) in [
        ("[1]", "IndexOf"),
        ("[1]", "FindIndex"),
        ("'abc'", "IndexOf"),
        ("'abc'", "LastIndexOf"),
    ] {
        let temp = TempDirectory::new("native-search-option-signature");
        let source = format!("program Demo; begin discard {receiver}.{method}(\nend.");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find(&format!("{method}(")).expect("call") + method.len() + 1;
        let help = service
            .signature_help(&path, cursor)
            .expect("signature query")
            .value
            .expect("native signature");
        assert!(
            help.signature.label.ends_with(": Option of integer"),
            "{help:#?}"
        );
    }
}

#[test]
fn pop_result_fields_retain_element_types_in_recovered_chains() {
    assert_string_chain(
        "",
        "ArrayPopResult of string",
        "['x'].Pop()",
        "Value.Remaining[0]",
    );
    assert_string_chain("", "array of string", "['x']", "Value.Pop().Remaining[0]");
    assert_string_chain("", "array of string", "['x']", "Value.Pop().Value");
}

#[test]
fn pop_result_completion_exposes_named_fields_with_specialized_types() {
    let temp = TempDirectory::new("native-pop-fields");
    let source = "program Demo; begin const Popped := ['x'].Pop(); discard Popped.\nend.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("Popped.\n").expect("result") + "Popped.".len();
    let candidates = service
        .completions(&path, cursor)
        .expect("completion")
        .value;
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.detail.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["field Remaining: array of string", "field Value: string"])
    );
}

#[test]
fn nested_result_success_chain_completes_every_string_operation() {
    assert_string_chain(
        "",
        "Result of (Result of (string, integer), boolean)",
        "Ok(Ok('x'))",
        "Value.Unwrap().Unwrap()",
    );
}

#[test]
fn nested_result_error_type_preserves_success_completion() {
    assert_string_chain(
        "",
        "Result of (string, Result of (integer, boolean))",
        "Ok('x')",
        "Value.Unwrap()",
    );
}

#[test]
fn multiple_result_levels_and_mixed_containers_preserve_completion() {
    for (ty, value, receiver) in [
        (
            "Result of (Result of (Result of (string, integer), boolean), real)",
            "Ok(Ok(Ok('x')))",
            "Value.Unwrap().Unwrap().Unwrap()",
        ),
        (
            "Result of (Option of string, integer)",
            "Ok(Some('x'))",
            "Value.Unwrap().Unwrap()",
        ),
        (
            "array of Result of (Result of (string, integer), boolean)",
            "[Ok(Ok('x'))]",
            "Value[0].Unwrap().Unwrap()",
        ),
        (
            "dict of string to Result of (Result of (string, integer), boolean)",
            "['key': Ok(Ok('x'))]",
            "Value['key'].Unwrap().Unwrap()",
        ),
    ] {
        assert_string_chain("", ty, value, receiver);
    }
}

#[test]
fn aliases_resolve_at_each_container_level() {
    assert_string_chain(
        "type Text = string; type Inner = Result of (Text, integer); type Outer = Result of (Inner, boolean); type Alias = Outer;",
        "Alias",
        "Ok(Ok('x'))",
        "Value.Unwrap().Unwrap()",
    );
}

#[test]
fn named_and_anonymous_callbacks_keep_nested_result_outputs() {
    let declarations = "function Wrap(X: integer): Result of (Result of (string, integer), boolean); begin return Ok(Ok('x')); end function;";
    for callback in [
        "Wrap",
        "F := Wrap",
        "function(X: integer): Result of (Result of (string, integer), boolean) begin return Ok(Ok('x')); end function",
    ] {
        let receiver = format!("Value.Map({callback})[0].Unwrap().Unwrap()");
        assert_string_chain(declarations, "array of integer", "[1]", &receiver);
    }
}

#[test]
fn callback_type_aliases_keep_nested_result_outputs() {
    assert_string_chain(
        "type Callback = function(X: integer): Result of (Result of (string, integer), boolean); function Wrap(X: integer): Result of (Result of (string, integer), boolean); begin return Ok(Ok('x')); end function; const Transform: Callback := Wrap;",
        "array of integer",
        "[1]",
        "Value.Map(Transform)[0].Unwrap().Unwrap()",
    );
}

#[test]
fn callable_receivers_distinguish_function_values_from_call_results() {
    let declarations = "type Nested = Result of (Result of (string, integer), boolean); type Reader = function(): Nested; function Fetch(): Nested; begin return Ok(Ok('x')); end function; function Build(X: integer): Reader; begin return Fetch; end function;";
    for (receiver, string_result) in [
        ("Fetch().Unwrap().Unwrap()", true),
        ("Alias().Unwrap().Unwrap()", true),
        ("Direct().Unwrap().Unwrap()", true),
        ("Selected().Unwrap().Unwrap()", true),
        ("Readers[0]().Unwrap().Unwrap()", true),
        ("Fetch", false),
        ("Alias", false),
        ("Direct", false),
        ("Items.Map(Build)[0]", false),
        (
            "Items.Map(function(X: integer): Reader begin return Fetch; end function)[0]",
            false,
        ),
    ] {
        let temp = TempDirectory::new("callable-receiver");
        let source = format!(
            "program Demo; {declarations} begin const Alias: Reader := Fetch; const Direct: function(): Nested := Fetch; const Items: array of integer := [1]; const Readers: array of Reader := Items.Map(Build); const Selected: Reader := Readers[0]; discard {receiver}.\nend."
        );
        let (_, diagnostics) =
            parse_compilation_unit(&source.replace(".\nend.", ".Length();\nend."));
        assert!(diagnostics.is_empty(), "fixture: {diagnostics:#?}");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find(&format!("{receiver}.\n")).expect("query") + receiver.len() + 1;
        if string_result {
            assert_string_candidates(&mut service, &path, &source, cursor);
        } else {
            let candidates = service
                .completions(&path, cursor)
                .expect("completion")
                .value;
            assert!(
                candidates.is_empty(),
                "function value {receiver}: {candidates:#?}"
            );
        }
    }
}

#[test]
fn imported_aliases_resolve_in_their_own_unit_with_private_and_imported_types() {
    let temp = TempDirectory::new("native-imported-result-chain");
    let manifest = temp.write("demo.fpasprj", "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n");
    temp.write(
        "types.fpas",
        "unit Demo.Types; public type Text = string; end unit;",
    );
    temp.write("facade.fpas", "unit Demo.Facade; uses Demo.Types as T; type Inner = Result of (T.Text, integer); public type Outer = Result of (Inner, boolean); public function Make(): Outer; begin return Ok(Ok('x')); end function; end unit;");
    let source = "program Demo; uses Demo.Facade as F; type Inner = integer; begin const Value: F.Outer := F.Make(); discard Value.Unwrap().Unwrap().\nend.";
    let path = temp.write(
        "main.fpas",
        &source.replace("Unwrap().\n", "Unwrap().Length();\n"),
    );
    let mut service = LanguageService::load(&manifest);
    service
        .documents_mut()
        .open_document(&path, 1, source.to_owned())
        .expect("incomplete editor buffer");
    let cursor = source.find("Unwrap().\n").expect("chain") + "Unwrap().".len();
    assert_string_candidates(&mut service, &path, source, cursor);
}

#[test]
fn nested_error_types_are_retained_in_signature_help() {
    let temp = TempDirectory::new("native-result-error-signature");
    let source = "program Demo; begin const Value: Result of (string, Result of (string, integer)) := Ok('x'); discard Value.OrElse(\nend.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("OrElse(").expect("call") + "OrElse(".len();
    let help = service
        .signature_help(&path, cursor)
        .expect("signature query")
        .value
        .expect("signature");
    assert!(
        help.signature.parameters[0].contains("Err: Result of (string, integer)"),
        "{help:#?}"
    );
}

#[test]
fn cyclic_aliases_and_malformed_type_fragments_return_no_native_candidates() {
    for declarations in [
        "type A = B; type B = A; const Value: A := 1;",
        "const Value: Result of (string,) := Ok('x');",
    ] {
        let temp = TempDirectory::new("native-invalid-receiver");
        let source = format!("program Demo; {declarations} begin discard Value.\nend.");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find("Value.\n").expect("chain") + "Value.".len();
        assert!(
            service
                .completions(&path, cursor)
                .expect("completion")
                .value
                .is_empty(),
            "{source}"
        );
    }
}

fn assert_string_chain(declarations: &str, ty: &str, initializer: &str, receiver: &str) {
    let temp = TempDirectory::new("native-result-chain");
    let source = format!(
        "program Demo;\n{declarations}\nbegin\n  const Value: {ty} := {initializer};\n  discard {receiver}.\nend.\n"
    );
    let path = temp.write("main.fpas", &source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source
        .find(&format!("{receiver}.\n"))
        .expect("incomplete chain")
        + receiver.len()
        + 1;
    assert!(!parse_compilation_unit(&source).1.is_empty());
    assert_string_candidates(&mut service, &path, &source, cursor);
}

fn assert_string_candidates(
    service: &mut LanguageService,
    path: &std::path::Path,
    source: &str,
    cursor: usize,
) {
    let candidates = service.completions(path, cursor).expect("completion").value;
    let expected: BTreeSet<_> = fpas_sema::native_operations()
        .filter(|operation| operation.receiver.accepts(&Ty::String))
        .map(|operation| format!("string.{}", operation.name))
        .collect();
    let actual: BTreeSet<_> = candidates
        .iter()
        .map(|candidate| candidate.qualified_name.clone())
        .collect();
    assert_eq!(actual, expected, "{source}");
    assert!(candidates.iter().all(|candidate| {
        candidate.inline_documentation.is_some() && candidate.additional_edit.is_none()
    }));
}
