//! Integration tests for editor metadata generated from intrinsic standard APIs.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixtures use expect to keep editor assertions focused"
)]

mod support;

use std::path::Path;

use fpas_language_service::{LanguageService, RenameError, SymbolKind};
use support::TempDirectory;

fn intrinsic_std_fixture(source: &str) -> (TempDirectory, std::path::PathBuf, LanguageService) {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = TempDirectory::new("intrinsic-standard-library-api");
    let path = temp.write("intrinsic.fpas", source);
    let service =
        LanguageService::load_with_standard_library(temp.path(), &repository_root.join("lib"))
            .expect("repository standard library");
    (temp, path, service)
}

#[test]
fn intrinsic_std_hover_includes_markdown_and_parameter_documentation() {
    let source =
        "program IntrinsicHover;\n\nuses Std.Fs;\n\nbegin\n  ReadText('notes.txt')\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("ReadText").expect("ReadText call");

    let hover = service
        .hover(&path, offset)
        .expect("intrinsic hover")
        .value
        .expect("ReadText hover");

    assert_eq!(
        hover.documentation.as_deref(),
        Some(
            "Reads UTF-8 text.\n\nParameters:\n- `Path`: File or directory path processed by the operation."
        )
    );
}

#[test]
fn intrinsic_std_completion_resolves_lazy_markdown_documentation() {
    let source = "program IntrinsicCompletion;\n\nuses Std.Fs;\n\nbegin\n  Read\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("Read\n").expect("Read prefix") + "Read".len();

    let candidate = service
        .completions(&path, offset)
        .expect("intrinsic completion")
        .value
        .into_iter()
        .find(|candidate| candidate.qualified_name == "Std.Fs.ReadText")
        .expect("ReadText completion");
    let identity = candidate.documentation.expect("documentation identity");
    let documentation = service
        .completion_documentation(
            &identity.path,
            identity.declaration_offset,
            &identity.qualified_name,
        )
        .expect("completion documentation");

    assert!(
        documentation
            .as_deref()
            .is_some_and(|value| value.contains("- `Path`:")),
        "{documentation:?}"
    );
}

#[test]
fn intrinsic_std_completion_offers_the_required_unit_import() {
    let source = "program IntrinsicImport;\n\nbegin\n  ReadText\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("ReadText").expect("ReadText prefix") + "ReadText".len();

    let candidate = service
        .completions(&path, offset)
        .expect("intrinsic auto import")
        .value
        .into_iter()
        .find(|candidate| candidate.qualified_name == "Std.Fs.ReadText")
        .expect("ReadText auto import");

    assert_eq!(candidate.owner.as_deref(), Some("Std.Fs"));
    assert_eq!(
        candidate
            .additional_edit
            .expect("Std.Fs import edit")
            .new_text,
        "\n\nuses Std.Fs;"
    );
}

#[test]
fn intrinsic_std_definition_targets_the_editor_api_declaration() {
    let source = "program IntrinsicDefinition;\n\nuses Std.Console;\n\nbegin\n  var Value: Color := CrtColor(1)\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("Color :=").expect("Color type");

    let definitions = service
        .definitions(&path, offset)
        .expect("intrinsic definition")
        .value;

    assert_eq!(definitions.len(), 1, "{definitions:#?}");
    assert!(
        definitions[0]
            .path
            .ends_with(Path::new("lib/api/Std/Console.fpas")),
        "{:?}",
        definitions[0].path
    );
}

#[test]
fn intrinsic_std_editor_api_declarations_cannot_be_renamed() {
    let source =
        "program IntrinsicRename;\n\nuses Std.Fs;\n\nbegin\n  ReadText('notes.txt')\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("ReadText").expect("ReadText call");

    assert!(
        service
            .prepare_rename(&path, offset)
            .expect("intrinsic prepare rename")
            .value
            .is_none()
    );
    assert!(matches!(
        service.rename(&path, offset, "ReadDocument"),
        Err(RenameError::EditorApi)
    ));
}

#[test]
fn intrinsic_std_signature_help_uses_declared_parameters() {
    let source = "program IntrinsicSignature;\n\nuses Std.Fs;\n\nbegin\n  WriteText('notes.txt', 'hello')\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find(", 'hello'").expect("second argument") + 2;

    let help = service
        .signature_help(&path, offset)
        .expect("intrinsic signature help")
        .value
        .expect("WriteText signature");

    assert_eq!(help.signature.parameters, ["Path: string", "Text: string"]);
    assert!(
        help.documentation
            .as_deref()
            .is_some_and(|value| value.starts_with("Writes UTF-8 text,")),
        "{help:#?}"
    );
    assert_eq!(
        help.parameter_documentation,
        [
            Some("File or directory path processed by the operation.".to_owned()),
            Some("UTF-8 text processed by the operation.".to_owned())
        ]
    );
    assert_eq!(help.active_parameter, Some(1));
}

#[test]
fn intrinsic_std_keyword_enum_member_has_hover_and_definition() {
    let source = "program IntrinsicEnum;\n\nuses Std.Json;\n\nbegin\n  var Value: JsonValue := JsonValue.Array([])\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.rfind("Array").expect("Array variant");

    let hover = service
        .hover(&path, offset)
        .expect("keyword enum hover")
        .value
        .expect("Array hover");
    let definitions = service
        .definitions(&path, offset)
        .expect("keyword enum definition")
        .value;

    assert_eq!(
        hover.documentation.as_deref(),
        Some(
            "`Array` enum member.\n\nParameters:\n- `Items`: Array elements stored in the constructed value."
        )
    );
    assert_eq!(definitions.len(), 1, "{definitions:#?}");
    assert!(
        definitions[0]
            .path
            .ends_with(Path::new("lib/api/Std/Json.fpas"))
    );
}

#[test]
fn intrinsic_std_keyword_enum_constructor_has_signature_help() {
    let source = "program IntrinsicEnumSignature;\n\nuses Std.Json;\n\nbegin\n  var Value: JsonValue := JsonValue.Array([])\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("[]").expect("Array argument") + 1;

    let help = service
        .signature_help(&path, offset)
        .expect("Array signature help")
        .value
        .expect("Array constructor signature");

    assert_eq!(
        help.signature.parameters,
        ["Items: array of Std.Json.JsonValue"]
    );
    assert_eq!(
        help.parameter_documentation,
        [Some(
            "Array elements stored in the constructed value.".to_owned()
        )]
    );
}

#[test]
fn intrinsic_std_keyword_enum_member_is_completed() {
    let source =
        "program IntrinsicEnumCompletion;\n\nuses Std.Json;\n\nbegin\n  JsonValue.Arr\nend.\n";
    let (_temp, path, mut service) = intrinsic_std_fixture(source);
    let offset = source.find("Arr\n").expect("Array prefix") + "Arr".len();

    let candidates = service
        .completions(&path, offset)
        .expect("keyword enum completion")
        .value;

    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.qualified_name == "Std.Json.JsonValue.Array"),
        "{candidates:#?}"
    );
}

#[test]
fn intrinsic_std_editor_api_covers_the_semantic_registry() {
    let source = "program IntrinsicCatalog;\n\nbegin\nend.\n";
    let (_temp, _path, mut service) = intrinsic_std_fixture(source);
    let index = service
        .workspace_symbol_index()
        .expect("intrinsic workspace symbol index");

    for unit in fpas_sema::intrinsic_std_units() {
        for symbol in fpas_sema::intrinsic_std_symbols(unit) {
            assert!(
                !index.find_qualified(&symbol.qualified_name).is_empty(),
                "{} is absent from lib/api",
                symbol.qualified_name
            );
        }
    }
}

#[test]
fn every_intrinsic_callable_parameter_has_markdown_documentation() {
    let source = "program IntrinsicParameterCatalog;\n\nbegin\nend.\n";
    let (_temp, _path, mut service) = intrinsic_std_fixture(source);
    let index = service
        .workspace_symbol_index()
        .expect("intrinsic workspace symbol index");
    let mut parameter_count = 0;

    for location in index.all_locations() {
        if !location
            .path
            .to_string_lossy()
            .replace('\\', "/")
            .contains("/lib/api/Std/")
        {
            continue;
        }
        let Some(callable) = &location.symbol.callable else {
            continue;
        };
        if !matches!(
            location.symbol.kind,
            SymbolKind::Function
                | SymbolKind::Procedure
                | SymbolKind::Method
                | SymbolKind::EnumMember
        ) {
            continue;
        }
        let documentation = service
            .completion_documentation(
                &location.path,
                location.symbol.full_span.offset(),
                &location.symbol.qualified_name,
            )
            .expect("intrinsic callable documentation")
            .unwrap_or_else(|| panic!("{} has no documentation", location.symbol.qualified_name));
        for parameter in &callable.parameters {
            let name = parameter
                .split_once(':')
                .expect("typed callable parameter")
                .0
                .split_whitespace()
                .next_back()
                .expect("parameter name");
            assert!(
                documentation.contains(&format!("- `{name}`: ")),
                "{} does not document {name}:\n{documentation}",
                location.symbol.qualified_name
            );
            parameter_count += 1;
        }
    }

    assert!(
        parameter_count > 100,
        "unexpectedly small intrinsic API surface"
    );
}

#[test]
fn intrinsic_std_editor_api_is_valid_syntax_without_runtime_analysis() {
    let source = "program IntrinsicSyntax;\n\nbegin\nend.\n";
    let (_temp, _path, mut service) = intrinsic_std_fixture(source);
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let api = repository_root.join("lib/api/Std/Fs.fpas");

    let analysis = service
        .analyze_document(&api)
        .expect("intrinsic editor API syntax analysis");

    assert!(
        analysis.diagnostics().is_empty(),
        "{:?}",
        analysis.diagnostics()
    );
    assert!(analysis.semantic().is_none());
}
