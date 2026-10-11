//! Generic data parameters have isolated navigation, rename and token identities.

use super::*;
use fpas_language_service::SymbolKind;

#[test]
fn generic_parameters_are_lexical_names_without_qualified_members() {
    for (declaration, construction, receiver, member) in [
        (
            "type Box of Item = record Value: Item; end record;",
            "const B: Box of integer := Box(Value := 1);",
            "B",
            "Value",
        ),
        (
            "type Box of Item = record Value: Item; end record;",
            "const B: Box of integer := Box(Value := 1);",
            "Box",
            "Value",
        ),
        (
            "type Choice of Item = enum Found(Value: Item); end enum;",
            "const C: Choice of integer := Choice.Found(1);",
            "C",
            "Found",
        ),
        (
            "type Choice of Item = enum Found(Value: Item); end enum;",
            "const C: Choice of integer := Choice.Found(1);",
            "Choice",
            "Found",
        ),
    ] {
        let source =
            format!("program T; {declaration} begin {construction} discard {receiver}.Item; end.");
        let temp = TempDirectory::new("generic-parameter-member-scope");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let offset = source.rfind(".Item").expect("invalid member use") + 1;
        assert!(
            service
                .definitions(&path, offset)
                .expect("definition")
                .value
                .is_empty(),
            "{receiver}.Item must not resolve to a lexical type parameter"
        );
        let candidates = service
            .completions(&path, offset)
            .expect("completion")
            .value;
        assert!(candidates.iter().any(|candidate| candidate.label == member));
        assert!(!candidates.iter().any(|candidate| candidate.label == "Item"));
    }
}

#[test]
fn generic_data_parameters_have_lexical_editor_identity() {
    let source = "program T; type Item = string;
        type Box of Item: Comparable = record Value: Item; end record;
        type Choice of Item: Comparable = enum Found(Value: Item); end enum;
        begin const B: Box of integer := Box(Value := 1);
        const C: Choice of integer := Choice.Found(2);
        const Text: Item := 'outer'; discard B; discard C; discard Text; end.";
    for owner in ["Box", "Choice"] {
        let temp = TempDirectory::new("generic-parameter-identity");
        let path = temp.write("main.fpas", source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        assert!(
            service
                .analyze_document(&path)
                .expect("analysis")
                .diagnostics()
                .is_empty()
        );
        let declaration = source
            .find(&format!("{owner} of Item"))
            .expect("parameter declaration")
            + owner.len()
            + " of ".len();
        let usage = declaration
            + source[declaration..]
                .find("Value: Item")
                .expect("parameter use")
            + "Value: ".len();
        let definitions = service.definitions(&path, usage).expect("definition").value;
        assert_eq!(definitions.len(), 1, "{owner}: {definitions:#?}");
        assert_eq!(definitions[0].symbol.kind, SymbolKind::TypeParameter);
        assert_eq!(definitions[0].symbol.selection_span.offset(), declaration);
        assert_eq!(
            definitions[0].symbol.detail,
            "type parameter Item: Comparable"
        );
        let references = service
            .references(&path, usage, true)
            .expect("references")
            .value;
        let mut offsets: Vec<_> = references
            .iter()
            .map(|reference| reference.span.offset())
            .collect();
        offsets.sort_unstable();
        assert_eq!(offsets, [declaration, usage]);
        let tokens = service.semantic_tokens(&path).expect("tokens").value;
        for offset in [declaration, usage] {
            let token = tokens
                .iter()
                .find(|token| token.span.offset() == offset)
                .expect("parameter token");
            assert_eq!(token.kind, SemanticTokenKind::TypeParameter);
            assert_eq!(token.modifiers.declaration, offset == declaration);
        }
        let mut edits = service
            .rename(&path, usage, "Element")
            .expect("rename")
            .value;
        let mut edit_offsets: Vec<_> = edits.iter().map(|edit| edit.range.offset()).collect();
        edit_offsets.sort_unstable();
        assert_eq!(edit_offsets, [declaration, usage]);
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.offset()));
        let mut edited = source.to_owned();
        for edit in edits {
            edited.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
        }
        service
            .documents_mut()
            .open_document(&path, 1, edited.clone())
            .expect("renamed overlay");
        assert!(
            service
                .analyze_document(&path)
                .expect("renamed analysis")
                .diagnostics()
                .is_empty()
        );
        let outer_use = edited.find("Text: Item").expect("outer type use") + "Text: ".len();
        let outer = service
            .definitions(&path, outer_use)
            .expect("outer definition")
            .value;
        assert_eq!(outer.len(), 1);
        assert_eq!(outer[0].symbol.kind, SymbolKind::Type);
        assert_eq!(
            outer[0].symbol.selection_span.offset(),
            source.find("Item =").expect("outer declaration")
        );
    }
}

#[test]
fn parameter_declaration_spans_skip_owner_names_and_constraints() {
    for (declaration, body, parameter_name, marker) in [
        (
            "type Box of Box = record Value: Box; end record;",
            "const B: Box of integer := Box(Value := 1); discard B;",
            "Box",
            "of Box",
        ),
        (
            "type Choice of Choice = enum Found(Value: Choice); end enum;",
            "const C: Choice of integer := Choice.Found(1); discard C;",
            "Choice",
            "of Choice",
        ),
        (
            "type Pair of (Item: Comparable, Pair) = record Value: Pair; First: Item; end record;",
            "const P: Pair of (integer, integer) := Pair(Value := 1, First := 2); discard P;",
            "Pair",
            ", Pair",
        ),
    ] {
        let source = format!("program T; {declaration} begin {body} end.");
        let temp = TempDirectory::new("generic-parameter-declaration-span");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let analysis = service.analyze_document(&path).expect("analysis");
        assert!(
            analysis.diagnostics().is_empty(),
            "{source}: {:#?}",
            analysis.diagnostics()
        );
        let parameter =
            source.find(marker).expect("parameter") + marker.len() - parameter_name.len();
        let usage = source
            .find(&format!("Value: {parameter_name}"))
            .expect("parameter use")
            + "Value: ".len();
        let definitions = service.definitions(&path, usage).expect("definition").value;
        assert_eq!(definitions.len(), 1, "{definitions:#?}");
        assert_eq!(definitions[0].symbol.selection_span.offset(), parameter);
        let mut edits = service
            .rename(&path, usage, "Element")
            .expect("rename")
            .value;
        let mut offsets: Vec<_> = edits.iter().map(|edit| edit.range.offset()).collect();
        offsets.sort_unstable();
        assert_eq!(offsets, [parameter, usage]);
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.offset()));
        let mut edited = source;
        for edit in edits {
            edited.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
        }
        service
            .documents_mut()
            .open_document(&path, 1, edited)
            .expect("edited source");
        assert!(
            service
                .analyze_document(&path)
                .expect("edited analysis")
                .diagnostics()
                .is_empty()
        );
    }
}
