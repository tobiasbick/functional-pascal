//! Checked expression types, precise ranges and generic substitutions.

#![allow(
    clippy::expect_used,
    reason = "fixtures assert on source query results"
)]

mod support;

use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn expression_types_follow_the_original_ast_through_nested_expressions() {
    let temp = TempDirectory::new("expression-types");
    let source = "program Types;\n\
type Box of T = record\n  Value: T;\nend record;\n\
function Wrap<T>(Value: T): Box of T;\nbegin\n  return Box(Value := Value);\nend function;\n\
function Twice(Value: integer): integer;\nrequires Value >= 0;\nbegin\n  return Value * 2;\nend function;\n\
begin\n  const Values := [1, 2];\n  const Sum := Twice(Values[0]) + 3;\n\
  const Text := Wrap('hello').Value.ToUpper();\n\
  const Choice := if Sum > 0 then Some(Sum) else None end if;\n\
  const Read := function(): integer begin return Values[1]; end function;\n\
  discard Text;\n  discard Choice;\n  discard Read();\nend.\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service.analyze_document(&path).expect("analysis");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:?}",
        analysis.diagnostics()
    );
    for (needle, within, expected_type, expected_text) in [
        ("[1, 2]", 0, "array of integer", "[1, 2]"),
        (
            "Twice(Values[0]) + 3",
            "Twice(Values[0]) ".len(),
            "integer",
            "Twice(Values[0]) + 3",
        ),
        ("Twice(Values[0])", 0, "integer", "Twice(Values[0])"),
        ("Values[0]", 6, "integer", "Values[0]"),
        ("Values[0]", 7, "integer", "0"),
        ("Wrap('hello').Value", 0, "Box of string", "Wrap('hello')"),
        ("Wrap('hello').Value", 15, "string", "Wrap('hello').Value"),
        (".ToUpper()", 2, "string", "Wrap('hello').Value.ToUpper()"),
        (
            "if Sum > 0",
            0,
            "Option of integer",
            "if Sum > 0 then Some(Sum) else None end if",
        ),
        ("Sum > 0", 4, "boolean", "Sum > 0"),
        ("Some(Sum)", 0, "Option of integer", "Some(Sum)"),
        ("None end if", 0, "Option of integer", "None"),
        (
            "function(): integer",
            0,
            "function(): integer",
            "function(): integer begin return Values[1]; end function",
        ),
        ("Value >= 0", 6, "boolean", "Value >= 0"),
        ("Value * 2", 6, "integer", "Value * 2"),
    ] {
        let offset = source.find(needle).expect("needle") + within;
        let result = service.expression_type(&path, offset).expect("type query");
        let expression = result.value.expect(needle);
        assert_eq!(expression.type_name, expected_type, "{needle}");
        assert_eq!(
            &result.snapshot.source()[expression.span.offset()..expression.span.end()],
            expected_text,
            "{needle}"
        );
    }
    assert!(
        service
            .expression_type(&path, 0)
            .expect("outside expression")
            .value
            .is_none()
    );
}

#[test]
fn expression_types_follow_editor_snapshots_and_exclude_embedded_comments() {
    let temp = TempDirectory::new("expression-types-snapshots");
    let source = "program T; begin discard (1 + // explanation\n2); end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let plus = source.find('+').expect("operator");
    let original = service.expression_type(&path, plus).expect("integer type");
    assert_eq!(
        original.value.as_ref().expect("expression").type_name,
        "integer"
    );
    assert!(
        service
            .expression_type(&path, source.find("explanation").expect("comment"))
            .expect("comment query")
            .value
            .is_none()
    );

    let updated = source.replace("1 +", "1.0 +").replace("2)", "2.0)");
    service
        .documents_mut()
        .open_document(&path, 1, updated.clone())
        .expect("editor overlay");
    let result = service
        .expression_type(&path, updated.find('+').expect("operator"))
        .expect("real type");
    assert_eq!(result.value.expect("updated expression").type_name, "real");
    assert_eq!(result.snapshot.source(), updated);
    assert_eq!(original.snapshot.source(), source);
}
