//! Concrete field hover and completion for generic record instantiations.

use super::*;

#[test]
fn generic_record_fields_show_concrete_hover_and_completion_types() {
    let temp = TempDirectory::new("generic-record-members");
    let source = "program T; type Box of T = record public Value: T; end record; begin const Number: Box of integer := Box(Value := 1); const Text: Box of string := Box(Value := 'one'); discard Number.Value; discard Text.Value; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (receiver, ty) in [("Number", "integer"), ("Text", "string")] {
        let offset = source.find(&format!("discard {receiver}.Value")).unwrap()
            + "discard ".len()
            + receiver.len()
            + 1;
        let hover = service.hover(&path, offset).unwrap().value.unwrap();
        assert_eq!(hover.contents, format!("field Value: {ty}"));
        let completions = service.completions(&path, offset + 2).unwrap().value;
        let field = completions
            .iter()
            .find(|candidate| candidate.label == "Value")
            .unwrap();
        assert_eq!(field.detail, format!("field Value: {ty}"));
    }
}

#[test]
fn instantiated_record_fields_preserve_native_string_completion() {
    let temp = TempDirectory::new("generic-record-native-chain");
    let source = "program T; type Box of T = record public Value: T; end record; begin const Text: Box of string := Box(Value := 'one'); discard Text.Value.ToUp; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("ToUp").unwrap() + 4;
    let completions = service.completions(&path, cursor).unwrap().value;
    assert!(
        completions
            .iter()
            .any(|candidate| candidate.label == "ToUpper"),
        "{completions:?}"
    );
}

#[test]
fn imported_nested_generic_record_fields_keep_concrete_descriptions() {
    let temp = TempDirectory::new("generic-record-imported-members");
    let manifest = temp.write(
        "records.fpasprj",
        "[project]\nname = \"records\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    temp.write(
        "box.fpas",
        "unit Data.Boxes; public type Box of T = record public Value: T; end record; end unit;",
    );
    let source = "program T; uses Data.Boxes; begin const Nested: Box of Box of string := Box(Value := Box(Value := 'one')); discard Nested.Value.Value; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let offset = source.rfind(".Value").unwrap() + 1;
    let hover = service.hover(&path, offset).unwrap().value.unwrap();
    assert_eq!(hover.contents, "field Value: string");
    let candidates = service.completions(&path, offset + 2).unwrap().value;
    assert_eq!(
        candidates
            .iter()
            .find(|candidate| candidate.label == "Value")
            .unwrap()
            .detail,
        "field Value: string"
    );
}
