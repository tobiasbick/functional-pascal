//! Receiving annotations and payload paths own labels independently of implementations.

use super::*;

#[test]
fn cyclic_invalid_callable_initializer_keeps_navigation_bounded() {
    let source = "program T; begin const F := F(); discard F(Input := 1); end.";
    let temp = TempDirectory::new("cyclic-callable-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(!analysis.diagnostics().is_empty());
    let label = source.find("Input :=").expect("label");
    assert!(
        service
            .definitions(&path, label)
            .expect("definition")
            .value
            .is_empty()
    );
    assert!(service.rename(&path, label, "Value").is_err());
}

#[test]
fn indexed_factories_select_receiving_fields_before_return_signatures() {
    for (declarations, body) in [
        (
            "",
            "const Factories: array of Factory := [Make]; const F := Factories[0]();",
        ),
        (
            "type Box = record Work: Factory; end record;",
            "const Boxes: array of Box := [Box(Work := Make)]; const F := Boxes[0].Work();",
        ),
    ] {
        let source = format!(
            "program T; type Factory = function(): function(Input: integer): integer;
          function Double(N: integer): integer; begin return N * 2; end function;
          function Make(): function(Input: integer): integer; begin return Double; end function;
          {declarations} begin {body} discard F(Input := 1); end."
        );
        let temp = TempDirectory::new("indexed-factory-callable-labels");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let analysis = service.analyze_document(&path).expect("analysis");
        assert!(
            analysis.diagnostics().is_empty(),
            "{:#?}",
            analysis.diagnostics()
        );
        let label = source.find("Input :=").expect("label");
        let definitions = service.definitions(&path, label).expect("definition").value;
        assert_eq!(definitions.len(), 1, "{body}: {definitions:#?}");
        assert_eq!(
            definitions[0].symbol.selection_span.offset(),
            source
                .find("Input: integer")
                .expect("receiving return signature")
        );
        let edits = service.rename(&path, label, "Value").expect("rename").value;
        assert_eq!(edits.len(), 2, "{edits:#?}");
        assert!(edits.iter().any(|edit| edit.range.offset() == label));
    }
}

#[test]
fn global_alias_labels_own_calls_through_inferred_values() {
    let source = "program T; type Action = function(Input: integer): integer;
      const F: Action := function(Input: integer): integer begin return Input; end function;
      begin const G := F; discard G(Input := 1); end.";
    let temp = TempDirectory::new("global-callable-alias-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    let label = source.find("Input :=").expect("label");
    let declaration = source.find("Input: integer").expect("alias parameter");
    let definition = service.definitions(&path, label).expect("definition").value;
    assert_eq!(definition[0].symbol.selection_span.offset(), declaration);
    let alias_edits = service
        .rename(&path, declaration, "Value")
        .expect("alias rename")
        .value;
    assert_eq!(alias_edits.len(), 2, "{alias_edits:#?}");
    assert!(alias_edits.iter().any(|edit| edit.range.offset() == label));
    let implementation = source[declaration + 1..]
        .find("Input: integer")
        .expect("implementation parameter")
        + declaration
        + 1;
    let implementation_edits = service
        .rename(&path, implementation, "N")
        .expect("implementation rename")
        .value;
    assert_eq!(implementation_edits.len(), 2, "{implementation_edits:#?}");
    assert!(
        implementation_edits
            .iter()
            .all(|edit| edit.range.offset() != label)
    );
}

#[test]
fn extracted_callable_labels_keep_option_result_and_enum_field_roles() {
    for (ty, value, pattern, other) in [
        (
            "result of (A, B)",
            "Error(Value := Double)",
            "Error(Value := const F)",
            "Ok(Value := _)",
        ),
        (
            "result of (A, B)",
            "Ok(Value := Double)",
            "Ok(Value := const F)",
            "Error(Value := _)",
        ),
        (
            "Choice",
            "Choice.Pair(Right := Double, Left := Double)",
            "Choice.Pair(Right := const F, Left := _)",
            "",
        ),
        (
            "option of Choice",
            "Some(Value := Choice.Pair(Double, Double))",
            "Some(Value := Choice.Pair(_, const F))",
            "None",
        ),
    ] {
        let source = format!("program T; type A = function(Input: integer): integer;
          type B = function(Input: integer): integer;
          type Choice = enum Pair(Left: A; Right: B); end enum;
          function Double(N: integer): integer; begin return N * 2; end function;
          begin const Wrapped: {ty} := {value}; case Wrapped of when {pattern}: discard F(Input := 1); {} end case; end.", if other.is_empty() { String::new() } else { format!("when {other}: null;") });
        let temp = TempDirectory::new("callable-payload-role-labels");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let analysis = service.analyze_document(&path).expect("analysis");
        assert!(
            analysis.diagnostics().is_empty(),
            "{:#?}",
            analysis.diagnostics()
        );
        let label = source.find("Input :=").expect("call label");
        let expected = if pattern.starts_with("Ok") {
            source.find("Input: integer").expect("A parameter")
        } else {
            source.find("type B = function(").expect("B") + "type B = function(".len()
        };
        let definitions = service
            .definitions(&path, label)
            .expect("definitions")
            .value;
        assert_eq!(definitions.len(), 1, "{pattern}: {definitions:#?}");
        assert_eq!(
            definitions[0].symbol.selection_span.offset(),
            expected,
            "{pattern}"
        );
        let edits = service
            .rename(&path, expected, "Value")
            .expect("payload label rename")
            .value;
        assert_eq!(edits.len(), 2, "{pattern}: {edits:#?}");
        assert!(edits.iter().any(|edit| edit.range.offset() == label));
    }
}

#[test]
fn inline_aggregate_and_factory_return_signatures_own_callable_labels() {
    for (declarations, body, owner) in [
        (
            "",
            "const Jobs: array of function(Input: integer): integer := [Double]; discard Jobs[0](Input := 1);",
            "function(Input: integer)",
        ),
        (
            "function Make(): function(Input: integer): integer; begin return Double; end function;",
            "const F := Make(); discard F(Input := 1);",
            "function(Input: integer)",
        ),
        (
            "function Make(): function(): function(Input: integer): integer; begin return function(): function(Input: integer): integer begin return Double; end function; end function;",
            "const F := Make(); const G := F(); discard G(Input := 1);",
            "function(Input: integer)",
        ),
    ] {
        let source = format!(
            "program T; function Double(N: integer): integer; begin return N * 2; end function; {declarations} begin {body} end."
        );
        let temp = TempDirectory::new("callable-return-labels");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let analysis = service.analyze_document(&path).expect("analysis");
        assert!(
            analysis.diagnostics().is_empty(),
            "{:#?}",
            analysis.diagnostics()
        );
        let label = source.find("Input :=").expect("call label");
        let definitions = service.definitions(&path, label).expect("definition").value;
        assert_eq!(definitions.len(), 1, "{body}: {definitions:#?}");
        assert_eq!(
            definitions[0].symbol.selection_span.offset(),
            source.find(owner).expect("receiving type") + "function(".len()
        );
        let edits = service
            .rename(&path, label, "Value")
            .expect("return label rename")
            .value;
        assert!(edits.iter().any(|edit| edit.range.offset() == label));
    }
}

#[test]
fn later_outer_case_arm_labels_keep_the_outer_selector_after_nested_cases() {
    let source = "program T; type A = function(Input: integer): integer; type B = function(Input: integer): integer;
      function Double(N: integer): integer; begin return N * 2; end function;
      begin const Wrapped: result of (A, A) := Error(Double); const Inner: option of B := Some(Double);
      case Wrapped of when Ok(Value := const Earlier):
        case Inner of when Some(Value := const Nested): discard Nested(Input := 1); when None: null; end case;
        const ResultValue := case Inner of when Some(Value := const LastNested): LastNested(Input := 3); when None: 0; end case;
      when Error(Value := const Later): discard Later(Input := 2); end case; end.";
    let temp = TempDirectory::new("enclosing-case-callable-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    let label = source.find("Input := 2").expect("later outer label");
    let definitions = service.definitions(&path, label).expect("definition").value;
    assert_eq!(definitions.len(), 1, "{definitions:#?}");
    assert_eq!(
        definitions[0].symbol.selection_span.offset(),
        source.find("Input: integer").expect("outer alias")
    );
    let edits = service
        .rename(&path, label, "Value")
        .expect("outer rename")
        .value;
    assert_eq!(edits.len(), 2, "{edits:#?}");
    assert!(edits.iter().any(|edit| edit.range.offset() == label));
}

#[test]
fn nested_case_selector_expression_keeps_the_outer_binding_label_owner() {
    let source = "program T; type Action = function(Input: integer): integer;
      function Double(N: integer): integer; begin return N * 2; end function;
      begin const Flag: integer := 0; const Wrapped: option of Action := Some(Double);
      case case Flag of when 0: Wrapped; else Wrapped; end case of
        when Some(Value := const F): discard F(Input := 1); when None: null; end case; end.";
    let temp = TempDirectory::new("nested-case-selector-callable-labels");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    let label = source.find("Input :=").expect("label");
    let definitions = service.definitions(&path, label).expect("definition").value;
    assert_eq!(definitions.len(), 1, "{definitions:#?}");
    assert_eq!(
        definitions[0].symbol.selection_span.offset(),
        source.find("Input: integer").expect("receiving signature")
    );
}
