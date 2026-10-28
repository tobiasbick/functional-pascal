//! `fpas doc`: signature and documentation lookup through the language service.
//!
//! Documentation: `docs/pascal/program-structure/cli.md#looking-up-declarations`.

mod json;
mod text;

use std::io::Write;
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::{CLI_DOC_NOT_FOUND, PROJECT_STANDARD_LIBRARY_INVALID};
use fpas_language_service::{ApiEntryKind, ApiLookup, LanguageService, SymbolKind};

use crate::cli_input::{DocCliConfig, DocReportFormat};
use crate::cli_output::{CliFailure, Reporter};

/// Looks up one name and prints its matches; exits `1` when nothing matches.
pub(crate) fn doc_cli(
    config: &DocCliConfig,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let lookup = match lookup(config) {
        Ok(lookup) => lookup,
        Err(exit) => return exit(reporter),
    };
    let written = match config.report {
        Some(DocReportFormat::Json) => json::write(stdout, &lookup, &config.cwd),
        None => text::write(stdout, &lookup, &config.cwd),
    };
    if let Err(error) = written {
        return reporter.failure(&crate::cli_output::write_failure("doc output", &error));
    }
    if lookup.matches.is_empty() {
        return reporter.failure(&not_found(&lookup));
    }
    0
}

type Failure = Box<dyn FnOnce(&mut Reporter<'_>) -> i32>;

fn lookup(config: &DocCliConfig) -> Result<ApiLookup, Failure> {
    let library = match &config.standard_library {
        Some(root) => Some(root.clone()),
        None => crate::standard_library::default_standard_library_root().map_err(|message| {
            failure(
                CliFailure::new(PROJECT_STANDARD_LIBRARY_INVALID, message)
                    .with_help("Pass `--std-lib <directory>` containing `stdlib.fpasprj`."),
            )
        })?,
    };
    let mut service = match library {
        Some(root) => LanguageService::load_with_standard_library(&config.input, &root)
            .map_err(service_failure)?,
        None => LanguageService::load(&config.input),
    };
    let catalog = service.api_catalog().map_err(service_failure)?;
    Ok(catalog.lookup(&config.query))
}

fn failure(failure: CliFailure) -> Failure {
    Box::new(move |reporter| reporter.failure(&failure))
}

fn service_failure(error: fpas_language_service::LanguageServiceError) -> Failure {
    let records = error.diagnostics();
    Box::new(move |reporter| {
        reporter.records(&records);
        1
    })
}

fn not_found(lookup: &ApiLookup) -> CliFailure {
    let closest = if lookup.suggestions.is_empty() {
        String::new()
    } else {
        let names = lookup
            .suggestions
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Closest names: {names}. ")
    };
    CliFailure::new(
        CLI_DOC_NOT_FOUND,
        format!(
            "No declaration or built-in operation is named `{}`.",
            lookup.query
        ),
    )
    .with_help(format!(
        "{closest}Qualify a routine with its unit (`fpas doc Std.Io.Fs.ReadText`) and an operation with its type (`fpas doc string.TrimLeft`); `fpas doc Std.Io.Fs` lists a unit."
    ))
}

/// Stable lowercase name of a match category, shared by text and JSON output.
fn kind_name(kind: ApiEntryKind) -> &'static str {
    match kind {
        ApiEntryKind::Operation => "operation",
        ApiEntryKind::Factory => "factory",
        ApiEntryKind::Declaration(kind) => match kind {
            SymbolKind::Program => "program",
            SymbolKind::Unit => "unit",
            SymbolKind::ImportAlias => "import alias",
            SymbolKind::Constant => "constant",
            SymbolKind::Variable => "variable",
            SymbolKind::Type => "type",
            SymbolKind::Enum => "enum",
            SymbolKind::Function => "function",
            SymbolKind::Procedure => "procedure",
            SymbolKind::Method => "method",
            SymbolKind::TypeParameter => "type parameter",
            SymbolKind::Parameter => "parameter",
            SymbolKind::Field => "field",
            SymbolKind::EnumMember => "enum member",
            SymbolKind::LoopVariable => "loop variable",
        },
    }
}

/// Shows a declaring path relative to the working directory when it lies below it.
fn display_path(path: &Path, cwd: &Path) -> PathBuf {
    path.strip_prefix(cwd)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}
