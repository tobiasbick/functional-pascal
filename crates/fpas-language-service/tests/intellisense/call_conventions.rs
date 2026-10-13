//! Static labels, indexed receiving signatures and complete-list completion.

use super::*;

#[test]
fn indexed_record_callable_fields_keep_static_labels_and_source_owners() {
    let source =
        "program T; type Box = record Callback: function(Input: integer): integer; end record;
      function Double(N: integer): integer; begin return N * 2; end function;
      begin const Boxes: array of Box := [Box(Callback := Double)];
      discard Boxes[0].Callback(Input := 1); end.";
    let temp = TempDirectory::new("indexed-record-callable-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    let label = source.find("Input :=").expect("call label");
    let help = service
        .signature_help(&path, label + "Input := 1".len())
        .expect("help")
        .value
        .expect("signature");
    assert_eq!(help.signature.parameters, ["Input: integer"]);
    assert_eq!(help.active_parameter, Some(0));
    let completions = service.completions(&path, label).expect("completion").value;
    assert_eq!(completions.len(), 1, "{completions:#?}");
    assert_eq!(completions[0].insert_text, "Input");
    let definitions = service.definitions(&path, label).expect("definition").value;
    assert_eq!(definitions.len(), 1, "{definitions:#?}");
    assert_eq!(
        definitions[0].symbol.selection_span.offset(),
        source.find("Input: integer").expect("field signature")
    );
    let edits = service.rename(&path, label, "Value").expect("rename").value;
    assert_eq!(edits.len(), 2, "{edits:#?}");
    assert!(edits.iter().any(|edit| edit.range.offset() == label));
}

#[test]
fn named_callable_and_pattern_help_uses_visible_static_labels() {
    let source = "program T; type Action = function(Input: integer): integer;
        function Double(N: integer): integer; begin return N * 2; end function;
        type Pair = enum Values(First: integer; Second: integer); end enum;
        begin const F: function(Value: integer): integer := Double;
          const G := Double; const A: Action := Double;
          discard F(Value := 3); discard G(N := 4); discard A(Input := 5);
          const P: Pair := Pair.Values(1, 2);
          discard P is Pair.Values(Second := 2, First := _);
          discard Some(Value := 3) is Some(Value := _); end.";
    let temp = TempDirectory::new("static-callable-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    for (call, parameter, active) in [
        ("F(Value := 3", "Value: integer", 0),
        ("G(N := 4", "N: integer", 0),
        ("A(Input := 5", "Input: integer", 0),
        ("Second := 2", "Second: integer", 1),
        ("Some(Value := _", "Value: T", 0),
    ] {
        let cursor = source.find(call).expect("call") + call.len();
        let help = service
            .signature_help(&path, cursor)
            .expect("help")
            .value
            .expect("signature");
        assert_eq!(help.signature.parameters[active], parameter);
        assert_eq!(help.active_parameter, Some(active));
    }
    for (call, labels) in [
        ("F(", vec!["Value"]),
        ("G(", vec!["N"]),
        ("A(", vec!["Input"]),
        ("Pair.Values(Second", vec!["Second"]),
        ("Some(Value := _", vec!["Value"]),
    ] {
        let cursor = source.find(call).expect("call") + call.find('(').expect("open") + 1;
        let completions = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        let actual = completions
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(actual, labels, "{call}: {completions:#?}");
    }
}

#[test]
fn indexed_and_alias_callables_show_concrete_receiving_signatures() {
    let source = "program T; type Action = function(Input: integer): integer;
      function Double(N: integer): integer; begin return N * 2; end function;
      begin const F: Action := Double; const Jobs: array of Action := [F];
      discard F(Input := 2); discard Jobs[0](Input := 3);
      const Wrapped := Some(Value := F); case Wrapped of when Some(Value := const Extracted):
      discard Extracted(Input := 4); when None: null; end case; end.";
    let temp = TempDirectory::new("indexed-callable-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    for call in ["F(Input := 2", "Jobs[0](Input := 3", "Extracted(Input := 4"] {
        let cursor = source.find(call).expect("call") + call.len();
        let help = service
            .signature_help(&path, cursor)
            .expect("help")
            .value
            .expect("signature");
        assert_eq!(help.signature.parameters, ["Input: integer"]);
        assert!(help.signature.label.ends_with("): integer"), "{help:#?}");
        let argument = source.find(call).expect("call") + call.find('(').expect("open") + 1;
        assert_eq!(
            service
                .completions(&path, argument)
                .expect("completion")
                .value[0]
                .label,
            "Input"
        );
        let definitions = service
            .definitions(&path, argument)
            .expect("definition")
            .value;
        assert_eq!(definitions.len(), 1, "{call}: {definitions:#?}");
        assert_eq!(
            definitions[0].symbol.selection_span.offset(),
            source.find("Input: integer").expect("alias parameter")
        );
    }
}

#[test]
fn positional_argument_completion_keeps_value_candidates_without_named_labels() {
    let source = "program T; function Sub(Left: integer; Right: integer): integer; begin return Left - Right; end function; begin const Available := 2; discard Sub(1, Available); end.";
    let temp = TempDirectory::new("positional-value-completion");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let offset = source.rfind("Available").expect("argument");
    let completions = service
        .completions(&path, offset)
        .expect("completion")
        .value;
    assert!(
        completions
            .iter()
            .any(|candidate| candidate.label == "Available"),
        "{completions:#?}"
    );
    assert!(
        !completions
            .iter()
            .any(|candidate| candidate.insert_text == "Right := "
                || candidate.insert_text == "Left := "),
        "{completions:#?}"
    );
}
