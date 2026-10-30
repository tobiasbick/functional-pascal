//! Checked local inference in declaration hover, uses, and editor completions.

#![allow(
    clippy::expect_used,
    reason = "editor fixtures assert on query results"
)]

use crate::support;

use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn inferred_subranges_retain_the_declared_type_for_hover_and_navigation() {
    let temp = TempDirectory::new("subrange-inference");
    let source = "program T; type Percent = 0..100; begin const P := Percent(50); const Found: boolean := P in Percent; discard P; discard Found; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    assert!(
        service
            .analyze_document(&path)
            .expect("analysis")
            .diagnostics()
            .is_empty()
    );
    let offset = source.find("discard P").expect("use") + "discard ".len();
    assert_eq!(
        service
            .hover(&path, offset)
            .expect("hover")
            .value
            .expect("binding")
            .contents,
        "const P: Percent"
    );
    let definition = service
        .type_definitions(&path, offset)
        .expect("type definition")
        .value;
    assert_eq!(definition.len(), 1, "{definition:?}");
    assert_eq!(definition[0].path, path);
    let type_offset = source.find("in Percent").expect("membership type") + "in ".len();
    let membership_definition = service
        .definitions(&path, type_offset)
        .expect("membership definition")
        .value;
    assert_eq!(membership_definition.len(), 1, "{membership_definition:?}");
    assert_eq!(membership_definition[0].path, path);
    let found_offset = source.find("discard Found").expect("Boolean use") + "discard ".len();
    assert_eq!(
        service
            .hover(&path, found_offset)
            .expect("membership hover")
            .value
            .expect("Boolean binding")
            .contents,
        "const Found: boolean"
    );
}

#[test]
fn inferred_types_appear_on_declarations_uses_and_completion() {
    let temp = TempDirectory::new("local-inference");
    let source = "program T; type UserId = distinct integer; function Read(): integer; begin return 42; end function; begin const Number := Read(); var Text := 'Ada'.ToUpper(); const Values := [1, 2]; const Index := ['a': 1]; const Id := UserId(Number); const Sum := Number + 1; const ReadLocal := function(): integer begin return Sum; end function; discard Number; discard Text; discard Values; discard Index; discard Id; discard Sum; discard ReadLocal; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    assert!(
        service
            .analyze_document(&path)
            .expect("analysis")
            .diagnostics()
            .is_empty()
    );
    for (name, detail) in [
        ("Number", "const Number: integer"),
        ("Text", "var Text: string"),
        ("Values", "const Values: array of integer"),
        ("Index", "const Index: dict of string to integer"),
        ("Id", "const Id: UserId"),
        ("Sum", "const Sum: integer"),
        ("ReadLocal", "const ReadLocal: function(): integer"),
    ] {
        for offset in [
            source.find(&format!("{name} :=")).expect("declaration"),
            source.find(&format!("discard {name};")).expect("use") + "discard ".len(),
        ] {
            assert_eq!(
                service
                    .hover(&path, offset)
                    .expect("hover")
                    .value
                    .expect("binding")
                    .contents,
                detail
            );
        }
        let completions = service
            .completions(&path, source.find("end.").expect("body end"))
            .expect("completion");
        assert!(
            completions
                .value
                .iter()
                .any(|entry| entry.label == name && entry.detail == detail),
            "{name}: {:?}",
            completions.value
        );
    }
}

#[test]
fn inferred_hover_follows_nested_scope_shadowing() {
    let temp = TempDirectory::new("inference-shadowing");
    let source = "program T; begin const Value := 1; begin const Value := 'inner'; discard Value; end; discard Value; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (offset, expected) in [
        (
            source.find("discard Value").expect("inner use"),
            "const Value: string",
        ),
        (
            source.rfind("discard Value").expect("outer use"),
            "const Value: integer",
        ),
    ] {
        assert_eq!(
            service
                .hover(&path, offset + "discard ".len())
                .expect("hover")
                .value
                .expect("binding")
                .contents,
            expected
        );
    }
}

#[test]
fn inferred_imported_conversion_retains_its_nominal_type_for_navigation() {
    let temp = TempDirectory::new("inference-import");
    temp.write("app.fpasprj", "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n");
    let unit = temp.write(
        "Domain.fpas",
        "unit Domain; public type UserId = distinct integer; end unit;",
    );
    let source = "program T; uses Domain as D; begin const Id := D.UserId(1); discard Id; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let offset = source.find("discard Id").expect("use") + "discard ".len();
    let definitions = service
        .type_definitions(&path, offset)
        .expect("type definition")
        .value;
    assert!(
        definitions
            .iter()
            .any(|definition| definition.path == unit && definition.symbol.name == "UserId"),
        "{definitions:?}"
    );
}

#[test]
fn inferred_imported_constructors_keep_concrete_hover_members_and_type_navigation() {
    let temp = TempDirectory::new("inference-constructions");
    let manifest = temp.write("app.fpasprj", "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n");
    let unit = temp.write("Domain.fpas", "unit Domain; public type Box of T = record public Value: T; end record; public type Lookup of T = enum Found(Value: T); Missing; end enum; public type TextLookup = Lookup of string; public function Make(): Box of string; begin return Box(Value := 'text'); end function; public function MakeMissing<T>(): Lookup of T; begin return Lookup.Missing; end function; end unit;");
    let source = "program T; uses Domain as D; begin const Boxed := D.Make(); const Found := D.Lookup.Found(Boxed); const Missing := D.TextLookup.Missing; const Contextual: D.Lookup of string := D.MakeMissing(); discard Boxed.Value; discard Found; discard Missing; discard Contextual; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:?}",
        analysis.diagnostics()
    );
    for (name, detail, type_name) in [
        ("Boxed", "const Boxed: domain.box of string", "Box"),
        (
            "Found",
            "const Found: domain.lookup of domain.box of string",
            "Lookup",
        ),
        (
            "Missing",
            "const Missing: domain.lookup of string",
            "Lookup",
        ),
        (
            "Contextual",
            "const Contextual: D.Lookup of string",
            "Lookup",
        ),
    ] {
        let offset = source
            .find(&format!("discard {name}"))
            .expect("binding use")
            + "discard ".len();
        assert_eq!(
            service
                .hover(&path, offset)
                .expect("hover")
                .value
                .expect("binding hover")
                .contents,
            detail
        );
        let definitions = service
            .type_definitions(&path, offset)
            .expect("type definition")
            .value;
        assert!(
            definitions
                .iter()
                .any(|definition| definition.path == unit && definition.symbol.name == type_name),
            "{name}: {definitions:?}"
        );
    }
    let field_offset =
        source.find("discard Boxed.Value").expect("field use") + "discard Boxed.".len();
    assert_eq!(
        service
            .hover(&path, field_offset)
            .expect("field hover")
            .value
            .expect("field")
            .contents,
        "field Value: string"
    );
    let candidates = service
        .completions(&path, field_offset + 2)
        .expect("field completion")
        .value;
    assert!(
        candidates.iter().any(
            |candidate| candidate.label == "Value" && candidate.detail == "field Value: string"
        ),
        "{candidates:?}"
    );
}
