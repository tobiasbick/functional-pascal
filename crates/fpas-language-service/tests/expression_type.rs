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

#[test]
fn checked_types_select_bindings_labels_callees_and_nominal_categories() {
    use fpas_language_service::{CancellationToken, SourceQueryOperation};
    let temp = TempDirectory::new("checked-selection");
    let source = "program Types;\n\
type Box of T = record\n  Value: T;\nend record;\n\
type Money = distinct integer;\n\
type Defaults = record\n  Value: integer := 1 + 2;\nend record;\n\
function Identity<T>(Value: T): T;\nbegin\n  return Value;\nend function;\n\
function Twice(Count: integer): integer;\nbegin\n  return Count * 2;\nend function;\n\
procedure Use(Value: integer);\nbegin\n  null;\nend procedure;\n\
var Global: Money := Money(1);\n\
begin\n  const Boxes := [Box(Value := 'abc')];\n\
  const Extract := Identity(Value := Boxes[0].Value).Length();\n\
  const Size := Boxes[0].Value.Length();\n\
  const Readers := [function(): string begin return 'read'; end function];\n\
  discard Readers[0]();\n\
  const Read := Twice;\n\
  for I: integer := 0 to 1 do\n    discard I;\n  end for;\n\
  case Some(1) of\n    when Some(const N): discard N;\n    when None: null;\n  end case;\n\
  Use(Value := 1);\nend.\n";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (needle, within, expected, kind, selected) in [
        ("Global: Money", 0, "Money", "distinct", "Global"),
        ("Boxes :=", 0, "array of Box of string", "array", "Boxes"),
        ("Value: T", 0, "T", "generic_parameter", "Value"),
        ("Count: integer", 0, "integer", "integer", "Count"),
        ("I: integer", 0, "integer", "integer", "I"),
        ("const N", 6, "integer", "integer", "N"),
        (
            "Identity(Value",
            0,
            "string",
            "string",
            "Identity(Value := Boxes[0].Value)",
        ),
        ("Value := Boxes", 0, "string", "string", "Value"),
        ("Value := 'abc'", 0, "string", "string", "Value"),
        (
            "Boxes[0].Value",
            0,
            "array of Box of string",
            "array",
            "Boxes",
        ),
        ("Boxes[0].Value", 6, "integer", "integer", "0"),
        ("Boxes[0].Value", 10, "string", "string", "Boxes[0].Value"),
        (
            ".Length()",
            2,
            "integer",
            "integer",
            "Identity(Value := Boxes[0].Value).Length()",
        ),
        (
            "Read :=",
            0,
            "function(Count: integer): integer",
            "callable",
            "Read",
        ),
        (
            "Boxes[0].Value.Length()",
            0,
            "array of Box of string",
            "array",
            "Boxes",
        ),
        (
            "Boxes[0].Value.Length()",
            9,
            "string",
            "string",
            "Boxes[0].Value",
        ),
        (
            "Boxes[0].Value.Length()",
            16,
            "integer",
            "integer",
            "Boxes[0].Value.Length()",
        ),
        ("Readers[0]()", 8, "integer", "integer", "0"),
        ("1 + 2", 0, "integer", "integer", "1"),
        ("1 + 2", 2, "integer", "integer", "1 + 2"),
        ("\nUse(Value", 5, "integer", "integer", "Value"),
    ] {
        let query = service
            .source_query(
                &path,
                source.find(needle).expect("needle") + within,
                SourceQueryOperation::Type,
                1000,
                &CancellationToken::new(),
            )
            .expect("query");
        assert!(
            query.reasons.is_empty(),
            "{needle}: {:?} {:?}",
            query.reasons,
            query.diagnostics
        );
        let result = query.expression_type.expect(needle);
        assert_eq!(result.type_name, expected, "{needle}");
        assert_eq!(result.kind, kind, "{needle}");
        assert_eq!(
            &source[result.span.offset()..result.span.end()],
            selected,
            "{needle}"
        );
        if needle == "Global: Money" {
            assert!(result.declaration.is_some());
        }
    }
    for (needle, within) in [("Box of T", 0), ("\nUse(Value", 1), ("Types;", 0)] {
        let query = service
            .source_query(
                &path,
                source.find(needle).expect("needle") + within,
                SourceQueryOperation::Type,
                1000,
                &CancellationToken::new(),
            )
            .expect("query");
        assert!(query.reasons.is_empty(), "{needle}: {:?}", query.reasons);
        assert!(query.expression_type.is_none(), "{needle}");
    }
}

#[test]
fn unresolved_inner_types_are_unavailable_without_inventing_dynamic() {
    use fpas_language_service::{CancellationToken, SourceQueryOperation};
    let temp = TempDirectory::new("type-holes");
    let source = "program T; begin discard []; end.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let result = service
        .source_query(
            &path,
            source.find("[]").expect("array"),
            SourceQueryOperation::Type,
            1000,
            &CancellationToken::new(),
        )
        .expect("query");
    assert!(result.expression_type.is_none());
    assert!(result.reasons.contains("type_unavailable"));
}
