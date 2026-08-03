#![allow(
    clippy::expect_used,
    reason = "IntelliSense fixtures use explicit source offsets and readable assertions"
)]

mod support;

use fpas_language_service::{
    CompletionKind, CompletionSource, LanguageService, SymbolKind, WorkspaceContext,
};
use fpas_parser::parse_compilation_unit;
use support::TempDirectory;

#[test]
fn completion_reports_member_metadata_and_replaces_the_complete_identifier() {
    let temp = TempDirectory::new("intellisense-members");
    let source = r#"program Complete;

type Counter = record
  public Amount: integer;
  Secret: integer;
end;

begin
  var Music: string := '𝄞';
  var Value: Counter := record Amount := 1; Secret := 2; end;
  var ResultValue: integer := Value.AmTail
end.
"#;
    let path = temp.write("complete.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("AmTail").expect("member prefix") + 2;

    let candidates = service
        .completions(&path, cursor)
        .expect("member completion")
        .value;
    let amount = candidates
        .iter()
        .find(|candidate| candidate.label == "Amount")
        .expect("Amount completion");

    assert_eq!(amount.kind, CompletionKind::Symbol(SymbolKind::Field));
    assert_eq!(amount.owner.as_deref(), Some("Complete.Counter"));
    assert_eq!(amount.detail, "field Amount: integer");
    assert_eq!(
        &source[amount.replacement_span.offset()..amount.replacement_span.end()],
        "AmTail"
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.label == "Secret")
    );
}

#[test]
fn completion_ignores_comments_and_strings_but_survives_recovered_source() {
    let temp = TempDirectory::new("intellisense-recovery");
    let source = r#"program Recovery;

type Counter = record
  public Amount: integer;
end;

begin
  var Value: Counter := record Amount := 1; end;
  // Value.Am
  var Text: string := 'Value.Am';
  Value.Am
end.
"#;
    let path = temp.write("recovery.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    for cursor in [
        source.find("Value.Am\n").expect("comment completion") + "Value.Am".len(),
        source.find("Value.Am'").expect("string completion") + "Value.Am".len(),
    ] {
        assert!(
            service
                .completions(&path, cursor)
                .expect("ignored completion context")
                .value
                .is_empty()
        );
    }

    let recovered = source.rfind("Value.Am").expect("recovered member") + "Value.Am".len();
    assert!(
        service
            .completions(&path, recovered)
            .expect("recovered completion")
            .value
            .iter()
            .any(|candidate| candidate.label == "Amount")
    );
}

#[test]
fn completion_excludes_shadowed_and_private_declarations_and_adds_keywords() {
    let temp = TempDirectory::new("intellisense-scope");
    let source = r#"program Scope;

var Value: string := 'global';

function Read(Value: integer): integer;
begin
  va
end;

begin
end.
"#;
    let path = temp.write("scope.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("  va\n").expect("keyword prefix") + "  va".len();

    let candidates = service
        .completions(&path, cursor)
        .expect("scoped completion")
        .value;

    assert_eq!(
        candidates
            .iter()
            .filter(|candidate| candidate.label.eq_ignore_ascii_case("Value"))
            .count(),
        1
    );
    assert!(candidates.iter().any(|candidate| {
        candidate.label == "var" && candidate.kind == CompletionKind::Keyword
    }));
}

#[test]
fn completion_includes_public_declarations_from_workspace_dependencies() {
    let temp = TempDirectory::new("intellisense-workspace");
    temp.write(
        "phase.fpasworkspace",
        "[workspace]\nname = \"completion\"\nmembers = [\"lib/lib.fpasprj\", \"app/app.fpasprj\"]\n",
    );
    temp.write(
        "lib/lib.fpasprj",
        "[project]\nname = \"completion-lib\"\nkind = \"library\"\n\n[exports]\nunits = [\"Completion.Library\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "lib/src/library.fpas",
        "unit Completion.Library;\n\npublic function GreetingFor(Name: string): string;\nbegin\n  return Name\nend;\n",
    );
    temp.write(
        "lib/src/hidden.fpas",
        "unit Completion.Hidden;\n\npublic function HiddenDependencyValue(): integer;\nbegin\n  return 1\nend;\n",
    );
    temp.write(
        "app/app.fpasprj",
        "[project]\nname = \"completion-app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nworkspace = [\"completion-lib\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let source = "program CompletionApp;\n\nuses Completion.Library;\n\nbegin\n  GreetingFor('workspace')\nend.\n";
    let path = temp.write("app/src/main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::load(temp.path()));
    let cursor = source.find("GreetingFor").expect("call");

    let candidates = service
        .completions(&path, cursor)
        .expect("workspace completion")
        .value;
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.label == "GreetingFor"),
        "{candidates:#?}"
    );

    let hidden_source = source.replace("GreetingFor('workspace')", "HiddenDependencyValue");
    service
        .documents_mut()
        .open_document(&path, 1, hidden_source.clone())
        .expect("open inaccessible completion query");
    let hidden_cursor = hidden_source
        .find("HiddenDependencyValue")
        .expect("hidden query")
        + "HiddenDependencyValue".len();
    assert!(
        !service
            .completions(&path, hidden_cursor)
            .expect("inaccessible completion")
            .value
            .iter()
            .any(|candidate| candidate.label == "HiddenDependencyValue")
    );
}

#[test]
fn auto_import_is_offered_only_for_one_public_declaration_and_preserves_formatting() {
    let temp = TempDirectory::new("intellisense-auto-import");
    let manifest = temp.write(
        "demo.fpasprj",
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/core.fpas",
        "unit Demo.Core;\n\npublic function Existing(): integer;\nbegin\n  return 1\nend;\n",
    );
    temp.write(
        "src/importable.fpas",
        "unit Demo.Importable;\n\n/// Returns the unique imported value.\npublic function UniqueValue(): integer;\nbegin\n  return 2\nend;\n\nfunction HiddenValue(): integer;\nbegin\n  return 3\nend;\n",
    );
    temp.write(
        "src/first.fpas",
        "unit Demo.First;\n\npublic function SharedValue(): integer;\nbegin\n  return 1\nend;\n",
    );
    temp.write(
        "src/second.fpas",
        "unit Demo.Second;\n\npublic function SharedValue(): integer;\nbegin\n  return 2\nend;\n",
    );
    let source = "program AutoImport;\n\nuses Demo.Core;\n\nbegin\n  var Value: integer := UniqueValue\nend.\n";
    let main = temp.write("src/main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let cursor = source.find("UniqueValue").expect("unresolved name") + "UniqueValue".len();

    let candidates = service
        .completions(&main, cursor)
        .expect("auto-import completion")
        .value;
    let unique = candidates
        .iter()
        .find(|candidate| candidate.label == "UniqueValue")
        .expect("unique auto-import");

    assert_eq!(unique.source, CompletionSource::AutoImport);
    let edit = unique.additional_edit.as_ref().expect("uses edit");
    let mut edited = source.to_owned();
    edited.replace_range(edit.span.offset()..edit.span.end(), &edit.new_text);
    assert!(
        edited.contains("uses Demo.Core, Demo.Importable;"),
        "{edited}"
    );
    let (unit, errors) = parse_compilation_unit(&edited);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        fpas_fmt::format_source(&edited, &unit).expect("matching source and AST"),
        edited
    );

    let documentation_key = unique
        .documentation
        .as_ref()
        .expect("lazy documentation key");
    assert_eq!(
        service
            .completion_documentation(
                &documentation_key.path,
                documentation_key.declaration_offset,
                &documentation_key.qualified_name,
            )
            .expect("completion documentation")
            .as_deref(),
        Some("Returns the unique imported value.")
    );

    for name in ["SharedValue", "HiddenValue"] {
        let query_source = source.replace("UniqueValue", name);
        service
            .documents_mut()
            .open_document(&main, 1, query_source.clone())
            .expect("open unresolved query");
        let query_cursor = query_source.find(name).expect("query name") + name.len();
        assert!(
            !service
                .completions(&main, query_cursor)
                .expect("negative auto-import completion")
                .value
                .iter()
                .any(|candidate| candidate.label == name)
        );
        service.documents_mut().close_document(&main);
    }
}

#[test]
fn signature_help_tracks_nested_multiline_generic_method_and_callable_value_arguments() {
    let temp = TempDirectory::new("intellisense-signatures");
    let source = r#"program Signatures;

type Counter = record
  public function Add(Self: Counter; Amount: integer; LabelText: string): integer;
  begin
    return Amount
  end;
end;

type Shape = enum
  Circle(Radius: real; Filled: boolean);
end;

function Sum(Left: integer; Right: integer): integer;
begin
  return Left + Right
end;

function Identity<T>(Value: T): T;
begin
  return Value
end;

procedure Outer();
  procedure Inner(Value: integer; Flag: boolean);
  begin
  end;
begin
  Inner(1, true)
end;

begin
  var CounterValue: Counter := record end;
  var Callback: function(Left: integer; Right: integer): integer := Sum;
  var A: integer := Sum(1, Sum(2, 3));
  var B: integer := CounterValue.Add(
    1,
    'two');
  var C: integer := Callback(1, 2);
  var D: integer := Identity(1);
  var E: Shape := Shape.Circle(2.0, true)
end.
"#;
    let path = temp.write("signatures.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let nested_cursor = source.find("Sum(2, 3)").expect("nested call") + "Sum(2, ".len();
    let nested = service
        .signature_help(&path, nested_cursor)
        .expect("nested signature")
        .value
        .expect("nested callable");
    assert_eq!(nested.signature.parameters.len(), 2);
    assert_eq!(nested.active_parameter, Some(1));

    let method_cursor = source.find("    'two'").expect("multiline argument");
    let method = service
        .signature_help(&path, method_cursor)
        .expect("method signature")
        .value
        .expect("method callable");
    assert_eq!(
        method.signature.parameters,
        ["Amount: integer", "LabelText: string"]
    );
    assert_eq!(method.active_parameter, Some(1));

    let nested_procedure_cursor = source
        .find("Inner(1, true)")
        .expect("nested procedure call")
        + "Inner(1, ".len();
    let nested_procedure = service
        .signature_help(&path, nested_procedure_cursor)
        .expect("nested procedure signature")
        .value
        .expect("nested procedure callable");
    assert!(
        nested_procedure
            .signature
            .label
            .starts_with("procedure Inner(")
    );
    assert_eq!(nested_procedure.active_parameter, Some(1));

    for (call, expected_label) in [
        ("Callback(1, 2)", "function Callback("),
        ("Identity(1)", "function Identity<T>("),
        ("Shape.Circle(2.0, true)", "Circle("),
    ] {
        let cursor = source.find(call).expect("call") + call.find('(').expect("parenthesis") + 1;
        let help = service
            .signature_help(&path, cursor)
            .expect("signature help")
            .value
            .expect("callable signature");
        assert!(help.signature.label.contains(expected_label), "{help:?}");
    }
}
