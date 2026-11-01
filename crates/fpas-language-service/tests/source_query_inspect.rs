//! Inspection context uses checked type bindings, exact source spans and explicit omissions.
#![allow(
    clippy::expect_used,
    reason = "inspection fixtures identify each required source fact"
)]

use crate::support::TempDirectory;
use fpas_language_service::LanguageService;
use fpas_language_service::{
    CancellationToken, InspectionSectionKind, SourceQueryOperation, SourceSession,
};

#[test]
fn annotation_bindings_retain_alias_identity_even_when_value_types_are_equal() {
    let temp = TempDirectory::new("inspect-type-binding");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    temp.write(
        "types.fpas",
        "unit Types; public type Imported = integer; end unit;",
    );
    let source = "program Main; uses Types as T; type First = integer; type Second = integer; function Echo(Value: Second; Imported: T.Imported): First; begin return Value; end function; begin end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let analysis = service.analyze_document(&path).expect("checked analysis");
    assert!(
        !analysis.diagnostics().iter().any(|d| d.is_error()),
        "{:?}",
        analysis.diagnostics()
    );
    let metadata = analysis.semantic().expect("semantic facts").metadata();
    let reference = |offset| {
        metadata
            .type_references
            .iter()
            .find(|((_, start), _)| *start == offset)
            .map(|(_, r)| r)
            .expect("annotation binding")
    };
    let first = reference(source.find("): First").expect("result annotation") + 3);
    let second = reference(source.find("Value: Second").expect("parameter annotation") + 7);
    assert_eq!(first.ty, second.ty);
    assert_ne!(first.declaration, second.declaration);
    assert_eq!(
        first.declaration.expect("first declaration").offset,
        source.find("First =").expect("first name")
    );
    assert_eq!(
        second.declaration.expect("second declaration").offset,
        source.find("Second =").expect("second name")
    );
    let imported = reference(source.find("T.Imported").expect("import annotation"));
    assert!(imported.declaration.is_none());
    assert_eq!(imported.qualified_name.as_deref(), Some("Types.Imported"));
}

#[test]
fn inspection_preserves_contracts_and_selects_only_direct_named_types() {
    let temp = TempDirectory::new("inspect-one-hop");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    temp.write("types.fpas", "unit Types; public type Leaf = record public Value: integer; end record; public type Holder = record Item: Leaf; end record; public type Alias = Holder; end unit;");
    let source = "program Main; uses Types as T;\n// Echoes the given value.\nfunction Echo(Value: T.Alias): T.Alias;\nrequires true;\nensures (Returned): true;\nbegin\n const Direct: T.Leaf := T.Leaf(Value := 1);\n return Value;\nend function;\nbegin end.";
    temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol("Main.Echo", SourceQueryOperation::Inspect, 100, &cancel)
        .expect("inspect");
    assert!(
        result.reasons.is_empty(),
        "{:?}; {:?}",
        result.reasons,
        result.diagnostics
    );
    let inspection = result.inspection.expect("inspection");
    assert_eq!(
        inspection
            .sections
            .iter()
            .map(|s| s.kind)
            .collect::<Vec<_>>(),
        [
            InspectionSectionKind::Signature,
            InspectionSectionKind::Implementation,
            InspectionSectionKind::SupportingType,
            InspectionSectionKind::SupportingType
        ]
    );
    let signature = &inspection.sections[0];
    assert!(signature.excerpts[0].text.contains("requires true;"));
    assert!(
        signature.excerpts[0]
            .text
            .contains("ensures (Returned): true;")
    );
    assert_eq!(
        signature.documentation.as_deref(),
        Some("Echoes the given value.")
    );
    assert_eq!(inspection.sections[2].qualified_name, "Types.Alias");
    assert_eq!(inspection.sections[3].qualified_name, "Types.Leaf");
    assert!(
        !inspection
            .sections
            .iter()
            .any(|s| s.qualified_name == "Types.Holder")
    );
    for section in &inspection.sections {
        for excerpt in &section.excerpts {
            let snapshot = section.snapshot.as_ref().expect("physical source");
            assert_eq!(
                &snapshot.source()[excerpt.span.offset()..excerpt.span.end()],
                excerpt.text
            );
        }
    }
}

#[test]
fn inspection_keeps_nested_routines_in_the_body_and_excludes_supporting_method_bodies() {
    let temp = TempDirectory::new("inspect-method-and-nesting");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let source = "program Main; type Box = record Value: integer; function Read(Self: Box): integer; begin return Self.Value + 87654321; end function; end record;\nfunction Outer(Value: Box): integer;\nfunction Inner(): integer; begin return Value.Read(); end function;\nbegin return Inner(); end function;\nbegin end.";
    temp.write("main.fpas", source);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol("Main.Outer", SourceQueryOperation::Inspect, 100, &cancel)
        .expect("inspect nested");
    assert!(
        result.reasons.is_empty(),
        "{:?}; {:?}",
        result.reasons,
        result.diagnostics
    );
    let inspection = result.inspection.expect("inspection");
    assert!(
        inspection.sections[1].excerpts[0]
            .text
            .starts_with("function Inner")
    );
    let support = inspection
        .sections
        .iter()
        .find(|s| s.kind == InspectionSectionKind::SupportingType)
        .expect("receiver type");
    assert!(
        support
            .excerpts
            .iter()
            .any(|e| e.text.contains("Read(Self"))
    );
    assert!(!support.excerpts.iter().any(|e| e.text.contains("87654321")));
    assert!(
        inspection
            .omissions
            .iter()
            .any(|o| o.reason == "method_bodies_excluded")
    );
    let method = session
        .query_symbol("Main.Box.Read", SourceQueryOperation::Inspect, 100, &cancel)
        .expect("method");
    assert!(method.reasons.is_empty(), "{:?}", method.reasons);
    assert!(
        method
            .inspection
            .expect("method inspection")
            .sections
            .iter()
            .any(|s| s.kind == InspectionSectionKind::SupportingType
                && s.reason == "receiver"
                && s.qualified_name == "Main.Box")
    );
}

#[test]
fn inspection_bounds_exact_unicode_excerpts_and_supporting_type_count() {
    let temp = TempDirectory::new("inspect-bounds");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n",
    );
    let types = (0..14)
        .map(|n| format!("type Item{n} = integer;\r\n"))
        .collect::<String>();
    let params = (0..14)
        .map(|n| format!("P{n}: Item{n}"))
        .collect::<Vec<_>>()
        .join("; ");
    temp.write("main.fpas", &format!("program Main;\r\n{types}function Large({params}): integer;\r\nbegin\r\n// {}\r\nreturn 1;\r\nend function;\r\nbegin end.", "😀".repeat(20000)));
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol("Main.Large", SourceQueryOperation::Inspect, 100, &cancel)
        .expect("bounded inspect");
    assert!(result.reasons.is_empty(), "{:?}", result.reasons);
    let inspection = result.inspection.expect("inspection");
    assert!(inspection.excerpt_bytes <= 32768);
    assert_eq!(inspection.supporting_types, 12);
    assert!(
        inspection
            .omissions
            .iter()
            .any(|o| o.reason == "supporting_type_limit")
    );
    assert!(
        inspection
            .omissions
            .iter()
            .any(|o| o.reason == "excerpt_limit")
    );
    let bytes = inspection
        .sections
        .iter()
        .flat_map(|s| &s.excerpts)
        .map(|e| e.text.len())
        .sum::<usize>();
    assert_eq!(bytes, inspection.excerpt_bytes);
    for section in &inspection.sections {
        for excerpt in &section.excerpts {
            assert_eq!(
                &section.snapshot.as_ref().expect("source").source()
                    [excerpt.span.offset()..excerpt.span.end()],
                excerpt.text
            );
        }
    }
}

#[test]
fn inspection_of_generic_recursive_records_keeps_nominal_and_shadowed_bindings() {
    let temp = TempDirectory::new("inspect-generic-types");
    let manifest = temp.write(
        "app.fpasprj",
        "[project]\nname='app'\nkind='library'\n[sources]\ninclude=['*.fpas']\n",
    );
    temp.write("generic.fpas", "unit Generic; type T = integer; public type Token = distinct integer; public type Choice = enum First; Second; end enum; public type Container of T = record public Value: T; public Next: option of Container of T; end record; public function Echo<T>(Values: array of Container of T; Flag: Choice; Id: Token): T; begin return Values[0].Value; end function; end unit;");
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&manifest, None, &[], &cancel).expect("scope");
    let result = session
        .query_symbol("Generic.Echo", SourceQueryOperation::Inspect, 100, &cancel)
        .expect("inspect generic");
    assert!(
        result.reasons.is_empty(),
        "{:?}; {:?}",
        result.reasons,
        result.diagnostics
    );
    let inspection = result.inspection.expect("inspection");
    assert_eq!(
        inspection
            .sections
            .iter()
            .filter(|s| s.kind == InspectionSectionKind::SupportingType)
            .map(|s| s.qualified_name.as_str())
            .collect::<Vec<_>>(),
        ["Generic.Container", "Generic.Choice", "Generic.Token"]
    );
    assert!(
        inspection
            .sections
            .iter()
            .all(|s| s.origin == fpas_language_service::DeclarationOrigin::Owned)
    );
}

#[test]
fn incomplete_inspection_does_not_label_source_facts_checked_or_issue_a_reference() {
    let temp = TempDirectory::new("inspect-incomplete");
    let path = temp.write(
        "main.fpas",
        "program Main; function Read(): integer; begin return Missing; end function; begin end.",
    );
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(&path, None, &[], &cancel).expect("scope");
    let result = session
        .query_position(&path, 23, SourceQueryOperation::Inspect, 100, &cancel)
        .expect("partial inspection");
    assert!(result.reasons.contains("analysis_errors"));
    assert!(
        result
            .target
            .expect("observed declaration")
            .target_ref
            .is_none()
    );
    let inspection = result.inspection.expect("partial sections");
    assert!(inspection.sections.iter().all(|s| !s.checked));
}
