//! Decision binders keep their lexical identities through navigation and rename.

use super::*;
use fpas_language_service::SymbolKind;

fn value_offset(source: &str, marker: &str) -> usize {
    source.find(marker).expect("unique occurrence marker")
        + marker.rfind("Value").expect("binding name in marker")
}

fn assert_identity(source: &str, declaration: &str, uses: &[&str]) {
    let temp = TempDirectory::new("decision-binding-identity");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{source}: {:#?}",
        analysis.diagnostics()
    );
    let declaration_offset = value_offset(source, declaration);
    let mut expected = vec![declaration_offset];
    for usage in uses {
        let offset = value_offset(source, usage);
        expected.push(offset);
        let definitions = service
            .definitions(&path, offset)
            .expect("definition")
            .value;
        assert_eq!(definitions.len(), 1, "{usage}: {definitions:#?}");
        assert_eq!(definitions[0].symbol.kind, SymbolKind::Constant);
        assert_eq!(
            definitions[0].symbol.selection_span.offset(),
            declaration_offset,
            "{usage}"
        );
    }
    expected.sort_unstable();
    let references = service
        .references(&path, declaration_offset, true)
        .expect("references")
        .value;
    let mut actual: Vec<_> = references
        .iter()
        .map(|reference| reference.span.offset())
        .collect();
    actual.sort_unstable();
    assert_eq!(actual, expected, "references for {declaration}");
    let mut edits = service
        .rename(&path, declaration_offset, "Renamed")
        .expect("rename")
        .value;
    let mut actual: Vec<_> = edits.iter().map(|edit| edit.range.offset()).collect();
    actual.sort_unstable();
    assert_eq!(actual, expected, "rename for {declaration}");
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.offset()));
    let mut edited = source.to_owned();
    for edit in edits {
        edited.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
    }
    service
        .documents_mut()
        .open_document(&path, 1, edited)
        .expect("renamed source");
    let analysis = service.analyze_document(&path).expect("renamed analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "rename {declaration}: {:#?}",
        analysis.diagnostics()
    );
}

#[test]
fn case_expression_binders_and_outer_constants_rename_independently() {
    let source = "program T; const Value: integer := 9;
        function Read(Input: Option of integer): integer;
        begin return case Input of when Some(const Value): Value; when None: 0; end case; end function;
        begin discard Read(Some(1)); discard Value; end.";
    assert_identity(source, "Some(const Value)", &["Some(const Value): Value;"]);
    assert_identity(source, "const Value: integer", &["discard Value;"]);
}

#[test]
fn pattern_bindings_start_after_the_tested_value_and_complete_pattern() {
    let source = "program T; const Value: integer := 9;
        type Pair = enum Both(First: integer; Second: integer); end enum;
        function Read(Input: Pair): integer;
        begin return case Input of when Pair.Both(const Value, Value): Value; when Pair.Both(_, _): Value; end case; end function;
        const Answer: integer := case Some(Value) of when Some(const Value): Value; when None: Value; end case;
        begin discard Read(Pair.Both(1, 9)); discard Answer; discard Value; end.";
    assert_identity(
        source,
        "Pair.Both(const Value,",
        &["Pair.Both(const Value, Value): Value;"],
    );
    assert_identity(
        source,
        "when Some(const Value)",
        &["when Some(const Value): Value;"],
    );
    assert_identity(
        source,
        "const Value: integer",
        &[
            "Pair.Both(const Value, Value)",
            "when Pair.Both(_, _): Value;",
            "case Some(Value)",
            "when None: Value;",
            "discard Value;",
        ],
    );
}

#[test]
fn sibling_and_nested_case_arms_keep_distinct_binding_identities() {
    let source = "program T; const Value: integer := 9;
        function Read(Input: Result of (Option of integer, integer)): integer;
        begin return case Input of
          when Ok(Some(const Value)) if Value > 0:
            case Some(Value) of when Some(const Value): Value; when None: Value; end case;
          when Ok(Some(const Value)): Value;
          when Ok(None): Value;
          when Error(const Value): Value;
        end case; end function;
        begin discard Read(Ok(Some(1))); discard Value; end.";
    assert_identity(
        source,
        "Ok(Some(const Value)) if",
        &["if Value >", "case Some(Value)", "when None: Value;"],
    );
    assert_identity(
        source,
        "when Some(const Value)",
        &["when Some(const Value): Value;"],
    );
    assert_identity(
        source,
        "when Ok(Some(const Value)):",
        &["when Ok(Some(const Value)): Value;"],
    );
    assert_identity(
        source,
        "Error(const Value)",
        &["Error(const Value): Value;"],
    );
    assert_identity(
        source,
        "const Value: integer",
        &["when Ok(None): Value;", "discard Value;"],
    );
}

#[test]
fn case_expression_bindings_are_extracted_in_every_value_position() {
    let decision = "case Input of when Some(const Value) if Value > 0: Value;
        when Some(const Value): Value; when None: Value; end case";
    let statements = [
        format!("return {decision};"),
        format!("const Chosen: integer := {decision}; return Chosen;"),
        format!("return Identity({decision});"),
        format!("return [({decision})][0];"),
        format!("return Box(Number := {decision}).Number;"),
        format!(
            "const Lookup: dict of string to integer := ['chosen': {decision}]; return Lookup['chosen'];"
        ),
        format!(
            "const Read: function(): integer := function(): integer begin return {decision}; end function; return Read();"
        ),
    ];
    for statements in statements {
        let source = format!(
            "program T; const Value: integer := 9;
            type Box = record Number: integer; end record;
            function Identity(Number: integer): integer; begin return Number; end function;
            function Choose(Input: Option of integer): integer; begin {statements} end function;
            begin discard Choose(Some(1)); discard Value; end."
        );
        assert_identity(
            &source,
            "Some(const Value) if",
            &["if Value >", "if Value > 0: Value;"],
        );
        assert_identity(
            &source,
            "when Some(const Value):",
            &["when Some(const Value): Value;"],
        );
        assert_identity(
            &source,
            "const Value: integer",
            &["when None: Value;", "discard Value;"],
        );
    }
}

#[test]
fn statement_case_patterns_and_nonbinding_conditions_keep_distinct_scopes() {
    let source = "program T; const Value: integer := 9;
        begin const Input: Option of integer := Some(1);
          case Input of when Some(const Value): discard Value; when None: discard Value; end case;
          if Input is Some(_) and Value > 0 then discard Value;
          elsif Input is Some(_) then discard Value; else discard Value; end if;
          while Input is Some(_) and Value > 0 do discard Value; break; end while;
          discard Value;
        end.";
    assert_identity(
        source,
        "case Input of when Some(const Value)",
        &["when Some(const Value): discard Value;"],
    );
    assert_identity(
        source,
        "const Value: integer",
        &[
            "when None: discard Value;",
            "and Value > 0 then",
            "then discard Value;\n",
            "then discard Value; else",
            "else discard Value;",
            "and Value > 0 do",
            "do discard Value;",
            "end while;\n          discard Value;",
        ],
    );
}

#[test]
fn rejected_is_patterns_never_create_editor_bindings() {
    for statements in [
        "if Input is Some(const Ghost) then discard Ghost; end if;",
        "while Input is Some(const Ghost) do discard Ghost; break; end while;",
        "const Hit: boolean := Input is Some(const Ghost); discard Ghost;",
        "const Hit: boolean := if Input is Some(const Ghost) then true else false end if; discard Ghost;",
    ] {
        let source = format!(
            "program T; begin const Input: Option of integer := Some(1); {statements} end."
        );
        let temp = TempDirectory::new("rejected-is-binding");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let analysis = service.analyze_document(&path).expect("analysis");
        assert!(
            analysis
                .diagnostics()
                .iter()
                .any(|error| error.code == fpas_diagnostics::codes::SEMA_IS_PATTERN_BINDING)
        );
        for offset in [
            source.find("Ghost").expect("pattern name"),
            source.rfind("Ghost").expect("use"),
        ] {
            assert!(
                service
                    .definitions(&path, offset)
                    .expect("definition")
                    .value
                    .is_empty()
            );
            assert!(matches!(
                service.rename(&path, offset, "Renamed"),
                Err(fpas_language_service::RenameError::NoSymbol)
            ));
            assert!(
                !service
                    .completions(&path, offset)
                    .expect("completion")
                    .value
                    .iter()
                    .any(|candidate| candidate.label == "Ghost")
            );
        }
    }
}
