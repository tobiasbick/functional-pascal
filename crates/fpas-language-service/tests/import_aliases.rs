//! Alias-qualified completion boundaries in nested argument expressions.

mod support;

use std::path::Path;

use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
use fpas_language_service::LanguageService;
use fpas_language_service::RenameError;
use support::TempDirectory;

#[test]
fn imported_member_completion_preserves_nested_argument_boundaries() {
    let temp = TempDirectory::new("alias-completion");
    let source = "program P; uses Std.Console as Console; uses Std.Fs as Files;
        begin Console.WriteLn(Files.ReadText('path')); end program;";
    let path = temp.write("main.fpas", source);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut service = LanguageService::load_with_standard_library(temp.path(), &root.join("lib"))
        .expect("standard library");
    let start = source.find("ReadText").expect("member");
    for offset in [start, start + 4, start + "ReadText".len()] {
        let candidates = service
            .completions(&path, offset)
            .expect("completion")
            .value;
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.label == "ReadText")
            .expect("qualified member completion");
        assert_eq!(candidate.qualified_name, "Std.Fs.ReadText");
        assert_eq!(candidate.replacement_span.offset(), start);
        assert_eq!(candidate.replacement_span.length(), "ReadText".len());
    }
}

#[test]
fn unknown_call_after_an_alias_reference_has_a_located_diagnostic() {
    let temp = TempDirectory::new("alias-diagnostic");
    temp.write("app.fpasprj", "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/*.fpas\"]\n");
    temp.write(
        "src/core.fpas",
        "unit Semantic.Core; public const ExistingText: string := 'ok'; end unit;",
    );
    temp.write("src/importable.fpas", "unit Semantic.Importable; public function UniqueValue(): integer; begin return 42; end function; end unit;");
    let source = r#"program SemanticHost;

uses Semantic.Core as Core;

begin
  const Music: string := '𝄞' + Core.ExistingText;
  const Value: integer := UniqueValue();
end program;
"#;
    let path = temp.write("src/main.fpas", source);
    let mut service = LanguageService::load(&path);
    let result = service
        .analyze_document_diagnostics(&path)
        .expect("diagnostics");
    assert!(result.failure().is_none());
    let diagnostics = result.document().diagnostics();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, SEMA_UNKNOWN_NAME);
    let span = diagnostics[0].span.expect("source span");
    assert_eq!(span.source_id(), 0);
    assert_eq!(span.offset(), source.find("UniqueValue()").expect("call"));
    assert_eq!(span.line(), 7);
}

#[test]
fn rename_cannot_shadow_an_import_alias_in_a_nested_scope() {
    let temp = TempDirectory::new("alias-rename");
    temp.write(
        "model.fpas",
        "unit App.Model; public const Answer: integer := 42; end unit;",
    );
    let source = r#"program P; uses App.Model as Model; function F(): integer;
        begin const Local: integer := 1; return Local; end function;
        begin const Value: integer := F(); end program;"#;
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load(&path);
    let error = service
        .rename(&path, source.find("Local:").expect("declaration"), "mOdEl")
        .expect_err("import aliases are reserved in every lexical scope");
    assert_eq!(
        error,
        RenameError::Conflict {
            name: "mOdEl".to_owned()
        }
    );
}

#[test]
fn procedure_completion_survives_each_named_statement_closer() {
    let temp = TempDirectory::new("named-closer-completion");
    for body in [
        "if true then null; end if;",
        "for I: integer := 0 to 1 do null; end for;",
        "while false do null; end while;",
        "case 1 of when 1: null; end case;",
        "var R: Point := Point(X := 1);",
        "var R: Point := Point(X := 1) with X := 2; end with;",
    ] {
        let source = format!(
            "program P; type Point = record X: integer; end record;
            procedure After(); begin null; end procedure;
            begin {body} After(); end program;"
        );
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::load(&path);
        let offset = source.rfind("After()").expect("statement call");
        let candidates = service
            .completions(&path, offset)
            .expect("completion")
            .value;
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.label == "After"),
            "{body}"
        );
    }
}
