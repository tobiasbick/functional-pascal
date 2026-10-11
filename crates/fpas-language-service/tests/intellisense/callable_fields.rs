//! Concrete generic callable fields retain their original navigation identity.

use super::*;

#[test]
fn parameterless_generic_callables_and_scalar_fields_keep_their_call_shapes() {
    let source = "program T; type Handlers of Item = record
        Read: function(): Item; Notify: procedure(); Value: Item; end record;
        function Read(): integer; begin return 1; end function;
        procedure Notify(); begin null; end procedure;
        begin const B: Handlers of integer := Handlers(Read := Read, Notify := Notify, Value := 1);
          discard B.Read(); B.Notify(); discard B.Value; end.";
    let temp = TempDirectory::new("generic-callable-empty-parameters");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    for (call, label) in [
        ("B.Read(", "function Read(): integer"),
        ("B.Notify(", "procedure Notify()"),
    ] {
        let cursor = source.find(call).expect("call") + call.len();
        let help = service
            .signature_help(&path, cursor)
            .expect("help")
            .value
            .expect("parameterless signature");
        assert_eq!(help.signature.label, label);
        assert!(help.signature.parameters.is_empty());
        assert_eq!(help.active_parameter, None);
    }
    let edited = source.replace("discard B.Value;", "discard B.Value(1);");
    let cursor = edited.find("B.Value(").expect("invalid call") + "B.Value(".len();
    service
        .documents_mut()
        .open_document(&path, 1, edited)
        .expect("invalid scalar call");
    assert!(
        service
            .signature_help(&path, cursor)
            .expect("help")
            .value
            .is_none()
    );
}

#[test]
fn recovered_alias_cycles_have_no_member_signature_or_completion() {
    let source = "program T; type Alias = Alias;
        begin const B: Alias := 0; discard B.Callback(1); end.";
    let temp = TempDirectory::new("callable-alias-cycle");
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    assert!(
        !service
            .analyze_document(&path)
            .expect("analysis")
            .diagnostics()
            .is_empty()
    );
    let usage = source.find(".Callback").expect("member") + 1;
    assert!(
        service
            .definitions(&path, usage)
            .expect("definition")
            .value
            .is_empty()
    );
    assert!(
        !service
            .completions(&path, usage)
            .expect("completion")
            .value
            .iter()
            .any(|candidate| candidate.label == "Callback")
    );
    let cursor = source.find("Callback(").expect("call") + "Callback(".len();
    assert!(
        service
            .signature_help(&path, cursor)
            .expect("help")
            .value
            .is_none()
    );
}

#[test]
fn exported_alias_exposes_public_callables_and_hides_private_fields() {
    let temp = TempDirectory::new("generic-callable-private-owner");
    let manifest = temp.write("records.fpasprj", "[project]\nname = \"records\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n");
    let unit = "unit Data.Boxes;
        public type Hidden of Item = record
          public Callback: function(Input: Item): Item;
          Secret: Item;
        end record;
        public type Published = Hidden of string;
        function Identity(Input: string): string; begin return Input; end function;
        public function Make(): Published;
        begin return Hidden(Callback := Identity, Secret := 'private'); end function;
        end unit;";
    let unit_path = temp.write("boxes.fpas", unit);
    let source = "program T; uses Data.Boxes; begin const B: Published := Make(); discard B.Callback('ok'); end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let unit_analysis = service.analyze_document(&unit_path).expect("unit analysis");
    assert!(
        unit_analysis.diagnostics().is_empty(),
        "unit: {:#?}",
        unit_analysis.diagnostics()
    );
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:#?}",
        analysis.diagnostics()
    );
    let cursor = source.find("'ok'").expect("argument") + 1;
    let help = service
        .signature_help(&path, cursor)
        .expect("help")
        .value
        .expect("public callable signature");
    assert_eq!(
        help.signature.label,
        "function Callback(Input: string): string"
    );
    let usage = source.find(".Callback").expect("member") + 1;
    let definitions = service.definitions(&path, usage).expect("definition").value;
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].path, unit_path);
    assert_eq!(
        definitions[0].symbol.selection_span.offset(),
        unit.find("Callback:").expect("field declaration")
    );
    let completions = service.completions(&path, usage).expect("members").value;
    assert!(
        completions
            .iter()
            .any(|candidate| candidate.label == "Callback")
    );
    assert!(
        !completions
            .iter()
            .any(|candidate| candidate.label == "Secret")
    );
    let edited = source.replace("Make()", "Published()");
    let cursor = edited.find("Published()").expect("constructor") + "Published(".len();
    service
        .documents_mut()
        .open_document(&path, 1, edited)
        .expect("invalid construction");
    assert!(
        service
            .signature_help(&path, cursor)
            .expect("constructor help")
            .value
            .is_none()
    );
}

#[test]
fn generic_callable_fields_keep_concrete_signatures_and_active_arguments() {
    let source = "program T;
        type Box of Item = record
          Callback: function(First: Item; Fallback: Item): Item;
          Update: procedure(var Value: Item; Replacement: Item);
        end record;
        function SelectValue(First: string; Fallback: string): string;
        begin return First; end function;
        procedure ReplaceValue(var Value: string; Replacement: string);
        begin Value := Replacement; end procedure;
        begin const B: Box of string := Box(Callback := SelectValue, Update := ReplaceValue);
          var Text: string := 'first';
          discard B.Callback('first', 'fallback');
          discard B.Callback('backup', 'primary');
          B.Update(var Text, 'next');
        end.";
    for (field, label, parameters, arguments, usages) in [
        (
            "Callback",
            "function Callback(First: string; Fallback: string): string",
            vec!["First: string", "Fallback: string"],
            vec![
                ("'first',", 0),
                ("'fallback'", 1),
                ("'backup'", 0),
                ("'primary'", 1),
            ],
            vec!["Callback :=", "Callback('first'", "Callback('backup'"],
        ),
        (
            "Update",
            "procedure Update(var Value: string; Replacement: string)",
            vec!["var Value: string", "Replacement: string"],
            vec![("var Text,", 0), ("'next'", 1)],
            vec!["Update :=", "Update(var Text"],
        ),
    ] {
        let temp = TempDirectory::new("generic-callable-field");
        let path = temp.write("main.fpas", source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let analysis = service.analyze_document(&path).expect("analysis");
        assert!(
            analysis.diagnostics().is_empty(),
            "{:#?}",
            analysis.diagnostics()
        );
        for (argument, active) in arguments {
            let cursor = source.find(argument).expect("argument marker") + 2;
            let help = service
                .signature_help(&path, cursor)
                .expect("signature help")
                .value
                .expect("callable signature");
            assert_eq!(help.signature.label, label);
            assert_eq!(help.signature.parameters, parameters);
            assert_eq!(help.active_parameter, Some(active));
        }
        let declaration = source
            .find(&format!("{field}:"))
            .expect("field declaration");
        let usage = source.find(&format!("B.{field}(")).expect("field call") + 2;
        let definitions = service
            .definitions(&path, usage)
            .expect("field definition")
            .value;
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].path, path);
        assert_eq!(definitions[0].symbol.kind, SymbolKind::Field);
        assert_eq!(definitions[0].symbol.selection_span.offset(), declaration);
        let expected_detail = label.replacen(&format!(" {field}("), "(", 1);
        let hover = service
            .hover(&path, usage)
            .expect("hover")
            .value
            .expect("field hover");
        assert_eq!(hover.contents, format!("field {field}: {expected_detail}"));
        let completions = service
            .completions(&path, usage + 2)
            .expect("completion")
            .value;
        assert_eq!(
            completions
                .iter()
                .find(|candidate| candidate.label == field)
                .expect("field completion")
                .detail,
            hover.contents
        );
        let mut expected = vec![declaration];
        expected.extend(
            usages
                .iter()
                .map(|marker| source.find(marker).expect("field occurrence")),
        );
        expected.sort_unstable();
        let mut edits = service
            .rename(&path, usage, "Changed")
            .expect("field rename")
            .value;
        let mut actual: Vec<_> = edits.iter().map(|edit| edit.range.offset()).collect();
        actual.sort_unstable();
        assert_eq!(actual, expected);
        assert!(edits.iter().all(|edit| edit.path == path));
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
            "{:#?}",
            analysis.diagnostics()
        );
    }
}

#[test]
fn imported_callable_fields_specialize_through_aliases_and_nested_records() {
    let unit = "unit Data.Boxes;
        public type Box of Item = record
          public Callback: function(Input: Item): Item;
          public Update: procedure(var Value: Item; Replacement: Item);
        end record;
        public type Outer of Item = record public Inner: Box of Item; end record;
        end unit;";
    for (import, qualifier) in [
        ("uses Data.Boxes;", ""),
        ("uses Data.Boxes as Boxes;", "Boxes."),
    ] {
        let source = format!("program T; {import}
            type Payload = Option of array of integer;
            type Concrete = {qualifier}Box of Payload;
            type Nested = {qualifier}Outer of Payload;
            function Identity(Input: Payload): Payload; begin return Input; end function;
            procedure Replace(var Value: Payload; Replacement: Payload); begin Value := Replacement; end procedure;
            begin const B: Concrete := {qualifier}Box(Callback := Identity, Update := Replace);
              const Container: Nested := {qualifier}Outer(Inner := B);
              var Data: Payload := Some([1, 2]);
              discard Container.Inner.Callback(Data);
              Container.Inner.Update(var Data, Some([3]));
            end.");
        for (field, label, parameters, argument) in [
            (
                "Callback",
                "function Callback(Input: Option of array of integer): Option of array of integer",
                vec!["Input: Option of array of integer"],
                "Callback(Data",
            ),
            (
                "Update",
                "procedure Update(var Value: Option of array of integer; Replacement: Option of array of integer)",
                vec![
                    "var Value: Option of array of integer",
                    "Replacement: Option of array of integer",
                ],
                "Update(var Data",
            ),
        ] {
            let temp = TempDirectory::new("generic-callable-imported-field");
            let manifest = temp.write("records.fpasprj", "[project]\nname = \"records\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n");
            let unit_path = temp.write("boxes.fpas", unit);
            let path = temp.write("main.fpas", &source);
            let mut service = LanguageService::load(&manifest);
            let analysis = service.analyze_document(&path).expect("analysis");
            assert!(
                analysis.diagnostics().is_empty(),
                "{source}: {:#?}",
                analysis.diagnostics()
            );
            let usage = source.find(&format!(".{field}(")).expect("field call") + 1;
            let cursor = source.find(argument).expect("argument") + field.len() + 3;
            let help = service
                .signature_help(&path, cursor)
                .expect("help")
                .value
                .expect("callable signature");
            assert_eq!(help.signature.label, label);
            assert_eq!(help.signature.parameters, parameters);
            assert_eq!(help.active_parameter, Some(0));
            let definitions = service.definitions(&path, usage).expect("definition").value;
            assert_eq!(definitions.len(), 1);
            assert_eq!(definitions[0].path, unit_path);
            let declaration = unit.find(&format!("{field}:")).expect("field declaration");
            assert_eq!(definitions[0].symbol.selection_span.offset(), declaration);
            let edits = service
                .rename(&path, usage, "Changed")
                .expect("rename")
                .value;
            let mut actual: Vec<_> = edits
                .iter()
                .map(|edit| (edit.path.clone(), edit.range.offset()))
                .collect();
            actual.sort();
            let mut expected = vec![
                (unit_path.clone(), declaration),
                (
                    path.clone(),
                    source
                        .find(&format!("{field} :="))
                        .expect("construction label"),
                ),
                (path.clone(), usage),
            ];
            expected.sort();
            assert_eq!(actual, expected);
            for (document, text) in [(&path, source.as_str()), (&unit_path, unit)] {
                let mut edited = text.to_owned();
                let mut document_edits: Vec<_> =
                    edits.iter().filter(|edit| &edit.path == document).collect();
                document_edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.offset()));
                for edit in document_edits {
                    edited.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
                }
                service
                    .documents_mut()
                    .open_document(document, 1, edited)
                    .expect("renamed overlay");
            }
            let analysis = service.analyze_document(&path).expect("renamed analysis");
            assert!(
                analysis.diagnostics().is_empty(),
                "{:#?}",
                analysis.diagnostics()
            );
        }
    }
}
