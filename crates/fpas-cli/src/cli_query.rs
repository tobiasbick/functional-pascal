//! Read-only JSON navigation queries backed by the shared language service.
//!
//! Documentation: `docs/pascal/tools/source-queries.md`.

use std::io::Write;
use std::path::Path;

use fpas_diagnostics::codes::{
    CLI_ARGUMENTS_INVALID, CLI_INPUT_UNSUPPORTED, PROJECT_STANDARD_LIBRARY_INVALID,
};
use fpas_diagnostics::{FileDiagnostic, SourceSpan};
use fpas_language_service::{DocumentSnapshot, LanguageService, LanguageServiceError};
use serde::Serialize;

use crate::cli_input::{QueryCliConfig, QueryKind};
use crate::cli_output::{CliFailure, Reporter};

/// Executes a single query and writes exactly one versioned JSON document on success.
pub(crate) fn query_cli(
    config: &QueryCliConfig,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let report = match query(config) {
        Ok(report) => report,
        Err(Failure::Cli(failure)) => return reporter.failure(&failure),
        Err(Failure::Service(error)) => {
            reporter.records(&error.diagnostics());
            return 1;
        }
        Err(Failure::Diagnostics(records)) => {
            reporter.records(&records);
            return 1;
        }
    };
    let result = serde_json::to_writer(&mut *stdout, &report)
        .map_err(std::io::Error::other)
        .and_then(|()| writeln!(stdout));
    match result {
        Ok(()) => 0,
        Err(error) => reporter.failure(&crate::cli_output::write_failure("query output", &error)),
    }
}

enum Failure {
    Cli(Box<CliFailure>),
    Service(Box<LanguageServiceError>),
    Diagnostics(Vec<FileDiagnostic>),
}

impl From<LanguageServiceError> for Failure {
    fn from(error: LanguageServiceError) -> Self {
        Self::Service(Box::new(error))
    }
}

impl From<CliFailure> for Failure {
    fn from(error: CliFailure) -> Self {
        Self::Cli(Box::new(error))
    }
}

#[derive(Serialize)]
struct Report {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    query: &'static str,
    position: Location,
    matches: Vec<Match>,
}

#[derive(Serialize)]
struct Location {
    path: String,
    line: usize,
    column: usize,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Match {
    Reference {
        path: String,
        range: Range,
        is_declaration: bool,
    },
    Definition {
        path: String,
        range: Range,
    },
    Type {
        r#type: String,
        range: Range,
    },
}

#[derive(Serialize)]
struct Range {
    start: Position,
    end: Position,
}

#[derive(Serialize)]
struct Position {
    line: usize,
    column: usize,
}

fn query(config: &QueryCliConfig) -> Result<Report, Failure> {
    let library = match &config.standard_library {
        Some(root) => Some(root.clone()),
        None => crate::standard_library::default_standard_library_root().map_err(|message| {
            Failure::from(
                CliFailure::new(PROJECT_STANDARD_LIBRARY_INVALID, message)
                    .with_help("Pass `--std-lib <directory>` containing `stdlib.fpasprj`."),
            )
        })?,
    };
    let input = config.project.as_deref().unwrap_or(&config.path);
    let mut service = match library {
        Some(root) => LanguageService::load_with_standard_library(input, &root)?,
        None => LanguageService::load(input),
    };
    if !service.workspace().issues().is_empty() {
        return Err(Failure::from(CliFailure::new(
            CLI_INPUT_UNSUPPORTED,
            service
                .workspace()
                .issues()
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        )));
    }
    let snapshot = service.snapshot(&config.path)?;
    let offset = source_offset(&snapshot, config.line, config.column).ok_or_else(|| {
        Failure::from(CliFailure::new(CLI_ARGUMENTS_INVALID, format!(
            "Position {}:{} is outside source `{}`.", config.line, config.column, config.path.display()
        )).with_help("Lines and Unicode-scalar columns start at 1; choose a position within the source line."))
    })?;
    // Explicit context must not silently discover an unrelated source project.
    if config.project.is_some()
        && service
            .workspace()
            .project_for_source(snapshot.path())
            .is_none()
    {
        return Err(outside_context(snapshot.path()));
    }
    let analysis = service.analyze_document(snapshot.path())?;
    let errors = analysis
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .cloned()
        .map(|diagnostic| FileDiagnostic::new(diagnostic, Some(snapshot.path().to_path_buf())))
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return Err(Failure::Diagnostics(errors));
    }

    let matches = match config.kind {
        QueryKind::Refs => service
            .references(snapshot.path(), offset, config.include_declaration)?
            .value
            .into_iter()
            .map(|reference| {
                Ok(Match::Reference {
                    path: display_path(&reference.path),
                    range: source_range(&reference.snapshot, reference.span)?,
                    is_declaration: reference.is_declaration,
                })
            })
            .collect::<Result<Vec<_>, Failure>>()?,
        QueryKind::Definition => {
            let definitions = service.definitions(snapshot.path(), offset)?.value;
            let mut matches = Vec::with_capacity(definitions.len());
            for definition in definitions {
                let target = service.snapshot(&definition.path)?;
                matches.push(Match::Definition {
                    path: display_path(&definition.path),
                    range: source_range(&target, definition.symbol.selection_span)?,
                });
            }
            matches
        }
        QueryKind::Type => service
            .expression_type(snapshot.path(), offset)?
            .value
            .into_iter()
            .map(|expression| {
                Ok(Match::Type {
                    r#type: expression.type_name,
                    range: source_range(&snapshot, expression.span)?,
                })
            })
            .collect::<Result<Vec<_>, Failure>>()?,
    };
    Ok(Report {
        schema_version: 1,
        query: config.kind.name(),
        position: Location {
            path: display_path(snapshot.path()),
            line: config.line,
            column: config.column,
        },
        matches,
    })
}

fn outside_context(path: &Path) -> Failure {
    Failure::from(CliFailure::new(
        CLI_INPUT_UNSUPPORTED,
        format!(
            "Source `{}` does not belong to the selected project context.",
            path.display()
        ),
    ))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn source_offset(snapshot: &DocumentSnapshot, line: usize, column: usize) -> Option<usize> {
    let range = snapshot.line_index().line_range(line.checked_sub(1)?)?;
    let text = snapshot.source().get(range.clone())?;
    let index = column.checked_sub(1)?;
    let byte_column = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(text.len()))
        .nth(index)?;
    Some(range.start + byte_column)
}

fn source_range(snapshot: &DocumentSnapshot, span: SourceSpan) -> Result<Range, Failure> {
    let position = |offset| {
        let position = snapshot.line_index().position(offset)?;
        let start = snapshot.line_index().line_start(position.line)?;
        Some(Position {
            line: position.line + 1,
            column: snapshot.source().get(start..offset)?.chars().count() + 1,
        })
    };
    let range = position(span.offset())
        .zip(position(span.end()))
        .map(|(start, end)| Range { start, end });
    range.ok_or_else(|| {
        Failure::from(CliFailure::new(
            CLI_INPUT_UNSUPPORTED,
            "The language service returned a range outside its source snapshot.",
        ))
    })
}
