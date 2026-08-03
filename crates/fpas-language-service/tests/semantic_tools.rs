#![expect(
    clippy::expect_used,
    reason = "semantic tooling fixtures use explicit source offsets and diagnostics"
)]

mod support;

use fpas_diagnostics::codes::{SEMA_UNKNOWN_NAME, SEMA_UNKNOWN_TYPE};
use fpas_language_service::{
    DiagnosticIdentity, LanguageService, SemanticTokenKind, WorkspaceContext,
    diagnostics_for_document,
};
use fpas_parser::parse_compilation_unit;
use support::TempDirectory;

#[test]
fn semantic_tokens_classify_every_supported_symbol_kind_and_modifier() {
    let temp = TempDirectory::new("semantic-token-kinds");
    let source = r#"unit Semantic.Sample;

public const Answer: integer := 42;
public var LabelText: string := 'sample';

public type
  Choice = enum
    First;
  end;
  Counter = record
    public Value: integer;
    public property Current: integer read GetCurrent;
    public event Changed: procedure() read ReadChanged write WriteChanged;
    public function Add<T>(Self: Counter; Amount: T): integer;
    begin
      return Self.Value
    end;
  end;

public function Identity<T>(Input: T): T;
begin
  var Local: T := Input;
  return Local
end;

public procedure Notify(MessageText: string);
begin
  mutable var CopyText: string := MessageText
end;
"#;
    let path = temp.write("sample.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let result = service.semantic_tokens(&path).expect("semantic tokens");
    let observed = result
        .value
        .iter()
        .map(|token| {
            (
                &source[token.span.offset()..token.span.end()],
                token.kind,
                token.modifiers,
            )
        })
        .collect::<Vec<_>>();

    for (name, kind) in [
        ("Semantic", SemanticTokenKind::Namespace),
        ("Counter", SemanticTokenKind::Type),
        ("Choice", SemanticTokenKind::Enum),
        ("T", SemanticTokenKind::TypeParameter),
        ("Input", SemanticTokenKind::Parameter),
        ("Local", SemanticTokenKind::Variable),
        ("Value", SemanticTokenKind::Field),
        ("Current", SemanticTokenKind::Property),
        ("Changed", SemanticTokenKind::Event),
        ("First", SemanticTokenKind::EnumMember),
        ("Identity", SemanticTokenKind::Function),
        ("Notify", SemanticTokenKind::Procedure),
        ("Add", SemanticTokenKind::Method),
        ("Answer", SemanticTokenKind::Constant),
    ] {
        assert!(
            observed
                .iter()
                .any(|(value, token_kind, _)| *value == name && *token_kind == kind),
            "missing {name:?} as {kind:?}: {observed:#?}"
        );
    }

    let answer = observed
        .iter()
        .find(|(name, kind, _)| *name == "Answer" && *kind == SemanticTokenKind::Constant)
        .expect("constant token");
    assert!(answer.2.declaration && answer.2.readonly && answer.2.public);
}

#[test]
fn semantic_tokens_follow_shadowing_and_return_partial_malformed_results() {
    let temp = TempDirectory::new("semantic-token-shadowing");
    let source = r#"program Shadowing;

var Value: integer := 1;

function Read(Value: integer): integer;
begin
  return Value
end;

begin
  var Broken: integer := Read(Value
end.
"#;
    let path = temp.write("shadowing.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let tokens = service
        .semantic_tokens(&path)
        .expect("partial semantic tokens")
        .value;
    let parameter_reference = source.find("return Value").expect("parameter reference") + 7;
    let global_reference = source.rfind("Value").expect("global reference");

    assert!(tokens.iter().any(|token| {
        token.span.offset() == parameter_reference && token.kind == SemanticTokenKind::Parameter
    }));
    assert!(tokens.iter().any(|token| {
        token.span.offset() == global_reference && token.kind == SemanticTokenKind::Variable
    }));
    assert!(
        tokens
            .windows(2)
            .all(|pair| pair[0].span.end() <= pair[1].span.offset())
    );
}

#[test]
fn unknown_name_action_adds_one_canonical_unambiguous_import() {
    let fixture = import_fixture("UniqueValue");
    let mut service = LanguageService::load(&fixture.manifest);
    let analysis = service
        .analyze_document(&fixture.main)
        .expect("project analysis");
    let diagnostic = diagnostics_for_document(&analysis)
        .iter()
        .find(|diagnostic| diagnostic.code == SEMA_UNKNOWN_NAME)
        .expect("unknown-name diagnostic");
    let identity = DiagnosticIdentity {
        code: diagnostic.code,
        message: diagnostic.message.clone(),
        span: diagnostic.span,
    };

    let actions = service
        .code_actions(&fixture.main, &identity)
        .expect("safe import action")
        .value;
    assert_eq!(actions.len(), 1, "expected one import action: {actions:#?}");
    let action = actions.first().expect("one import action");
    assert_eq!(action.title, "Import Actions.Importable");
    assert_eq!(action.diagnostic, identity);
    let mut edited = fixture.source.clone();
    let edit = &action.edits[0];
    edited.replace_range(edit.span.offset()..edit.span.end(), &edit.new_text);
    let (unit, diagnostics) = parse_compilation_unit(&edited);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert_eq!(
        fpas_fmt::format_source(&edited, &unit).expect("matching source and AST"),
        edited
    );
    assert!(edited.contains("uses Actions.Core, Actions.Importable;"));
}

#[test]
fn unknown_type_action_adds_one_canonical_unambiguous_import() {
    let temp = TempDirectory::new("semantic-type-code-action");
    let manifest = temp.write(
        "actions.fpasprj",
        "[project]\nname = \"actions\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/core.fpas",
        "unit Actions.Core;\n\npublic const Existing: integer := 1;\n",
    );
    temp.write(
        "src/types.fpas",
        "unit Actions.Types;\n\npublic type UniqueType = integer;\n",
    );
    let source =
        "program Actions;\n\nuses Actions.Core;\n\nbegin\n  var Value: UniqueType := 1\nend.\n";
    let main = temp.write("src/main.fpas", source);
    let mut service = LanguageService::load(&manifest);
    let analysis = service.analyze_document(&main).expect("project analysis");
    let diagnostics = diagnostics_for_document(&analysis);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == SEMA_UNKNOWN_TYPE)
        .expect("unknown-type diagnostic");
    let identity = DiagnosticIdentity {
        code: diagnostic.code,
        message: diagnostic.message.clone(),
        span: diagnostic.span,
    };

    let actions = service
        .code_actions(&main, &identity)
        .expect("safe type import action")
        .value;
    assert_eq!(
        actions.len(),
        1,
        "expected one type import action: {actions:#?}"
    );
    let action = actions.first().expect("one type import action");
    assert_eq!(action.title, "Import Actions.Types");
    let edit = &action.edits[0];
    let mut edited = source.to_owned();
    edited.replace_range(edit.span.offset()..edit.span.end(), &edit.new_text);
    let (unit, parse_diagnostics) = parse_compilation_unit(&edited);
    assert!(parse_diagnostics.is_empty(), "{parse_diagnostics:#?}");
    assert_eq!(
        fpas_fmt::format_source(&edited, &unit).expect("matching source and AST"),
        edited
    );
    assert!(edited.contains("uses Actions.Core, Actions.Types;"));
}

#[test]
fn import_action_rejects_a_stale_diagnostic_after_the_document_changes() {
    let fixture = import_fixture("UniqueValue");
    let mut service = LanguageService::load(&fixture.manifest);
    let analysis = service
        .analyze_document(&fixture.main)
        .expect("initial analysis");
    let diagnostic = diagnostics_for_document(&analysis)
        .iter()
        .find(|diagnostic| diagnostic.code == SEMA_UNKNOWN_NAME)
        .expect("initial unknown name");
    let stale = DiagnosticIdentity {
        code: diagnostic.code,
        message: diagnostic.message.clone(),
        span: diagnostic.span,
    };
    let changed = fixture.source.replace("UniqueValue", "DifferentValue");
    service
        .documents_mut()
        .open_document(&fixture.main, 2, changed)
        .expect("changed open document");
    assert!(
        service
            .code_actions(&fixture.main, &stale)
            .expect("stale action query")
            .value
            .is_empty()
    );
    service.documents_mut().close_document(&fixture.main);
}

#[test]
fn import_action_rejects_ambiguous_public_declarations() {
    let fixture = import_fixture("SharedValue");
    fixture.temp.write(
        "src/second.fpas",
        "unit Actions.Second;\n\npublic function SharedValue(): integer;\nbegin\n  return 2\nend;\n",
    );
    fixture.temp.write(
        "src/first.fpas",
        "unit Actions.First;\n\npublic function SharedValue(): integer;\nbegin\n  return 1\nend;\n",
    );
    let mut ambiguous_service = LanguageService::load(&fixture.manifest);
    let ambiguous_analysis = ambiguous_service
        .analyze_document(&fixture.main)
        .expect("ambiguous analysis");
    let ambiguous_diagnostic = diagnostics_for_document(&ambiguous_analysis)
        .iter()
        .find(|diagnostic| diagnostic.code == SEMA_UNKNOWN_NAME)
        .expect("unresolved ambiguous name");
    let ambiguous_identity = DiagnosticIdentity {
        code: ambiguous_diagnostic.code,
        message: ambiguous_diagnostic.message.clone(),
        span: ambiguous_diagnostic.span,
    };
    assert!(
        ambiguous_service
            .code_actions(&fixture.main, &ambiguous_identity)
            .expect("ambiguous action query")
            .value
            .is_empty()
    );
}

#[test]
fn import_action_rejects_an_inaccessible_private_declaration() {
    let fixture = import_fixture("PrivateValue");
    let mut service = LanguageService::load(&fixture.manifest);
    let analysis = service
        .analyze_document(&fixture.main)
        .expect("private declaration analysis");
    let diagnostic = diagnostics_for_document(&analysis)
        .iter()
        .find(|diagnostic| diagnostic.code == SEMA_UNKNOWN_NAME)
        .expect("unknown private name");
    let identity = DiagnosticIdentity {
        code: diagnostic.code,
        message: diagnostic.message.clone(),
        span: diagnostic.span,
    };

    assert!(
        service
            .code_actions(&fixture.main, &identity)
            .expect("private action query")
            .value
            .is_empty()
    );
}

#[test]
fn explanatory_parser_help_does_not_become_a_code_action() {
    let temp = TempDirectory::new("semantic-explanatory-help");
    let source = "program Broken;\n\nbegin\n  var Value integer := 1\nend.\n";
    let path = temp.write("broken.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let analysis = service
        .analyze_document(&path)
        .expect("syntax-only analysis");
    let diagnostic = diagnostics_for_document(&analysis)
        .first()
        .expect("parser diagnostic");
    let identity = DiagnosticIdentity {
        code: diagnostic.code,
        message: diagnostic.message.clone(),
        span: diagnostic.span,
    };

    assert!(
        service
            .code_actions(&path, &identity)
            .expect("explanatory action query")
            .value
            .is_empty()
    );
}

struct ImportFixture {
    temp: TempDirectory,
    manifest: std::path::PathBuf,
    main: std::path::PathBuf,
    source: String,
}

fn import_fixture(name: &str) -> ImportFixture {
    let temp = TempDirectory::new("semantic-code-actions");
    let manifest = temp.write(
        "actions.fpasprj",
        "[project]\nname = \"actions\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    temp.write(
        "src/core.fpas",
        "unit Actions.Core;\n\npublic function Existing(): integer;\nbegin\n  return 1\nend;\n",
    );
    temp.write(
        "src/importable.fpas",
        "unit Actions.Importable;\n\npublic function UniqueValue(): integer;\nbegin\n  return 42\nend;\n\nfunction PrivateValue(): integer;\nbegin\n  return 0\nend;\n",
    );
    let source = format!(
        "program Actions;\n\nuses Actions.Core;\n\nbegin\n  var Value: integer := {name}()\nend.\n"
    );
    let main = temp.write("src/main.fpas", &source);
    ImportFixture {
        temp,
        manifest,
        main,
        source,
    }
}
