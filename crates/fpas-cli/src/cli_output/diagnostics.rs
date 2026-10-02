//! Text or JSON Lines diagnostic output on stderr for `check`, `build`, `run` and `test`.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use fpas_diagnostics::{Diagnostic, FileDiagnostic};

use super::CliFailure;

/// Diagnostic stream format selected with `--diagnostics`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub(crate) enum DiagnosticFormat {
    /// Human-readable diagnostics plus progress and summary lines.
    #[default]
    Text,
    /// One JSON diagnostic record per stderr line and no other stderr output.
    Json,
}

impl DiagnosticFormat {
    /// Parses an explicit `--diagnostics` value.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    /// Finds `--diagnostics json` in raw arguments before program arguments, for
    /// reporting argument errors in the requested format.
    pub(crate) fn requested(args: &[String]) -> Self {
        args.iter()
            .take_while(|arg| *arg != "--")
            .collect::<Vec<_>>()
            .windows(2)
            .find(|pair| pair[0] == "--diagnostics")
            .and_then(|pair| Self::parse(pair[1]))
            .unwrap_or_default()
    }
}

/// Attributes a diagnostic to the source named by its span's source ID, else `fallback`.
pub(crate) fn locate(
    fallback: &Path,
    source_paths: Option<&[PathBuf]>,
    diagnostic: &Diagnostic,
) -> FileDiagnostic {
    let path = source_paths
        .and_then(|paths| {
            let index = usize::try_from(diagnostic.span?.source_id()).ok()?;
            paths.get(index)
        })
        .map_or_else(|| fallback.to_path_buf(), Clone::clone);
    FileDiagnostic::new(diagnostic.clone(), Some(path))
}

/// Writes diagnostics in the selected format and suppresses progress in JSON mode.
pub(crate) struct Reporter<'a> {
    format: DiagnosticFormat,
    stderr: &'a mut dyn Write,
    // Source text keyed by path, read lazily to resolve JSON end positions.
    sources: HashMap<PathBuf, Option<String>>,
}

impl<'a> Reporter<'a> {
    /// Creates a reporter that writes to `stderr`.
    pub(crate) fn new(format: DiagnosticFormat, stderr: &'a mut dyn Write) -> Self {
        Self {
            format,
            stderr,
            sources: HashMap::new(),
        }
    }

    /// Returns the selected format.
    pub(crate) fn format(&self) -> DiagnosticFormat {
        self.format
    }

    /// Writes one diagnostic record.
    pub(crate) fn record(&mut self, record: &FileDiagnostic) {
        let line = match self.format {
            DiagnosticFormat::Text => record.to_string(),
            DiagnosticFormat::Json => {
                let source = record.path.as_deref().and_then(|path| self.source(path));
                let path = record.path.as_deref().map(|path| path.to_string_lossy());
                match fpas_diagnostics::render_json(path.as_deref(), source, &record.diagnostic) {
                    Ok(line) => line,
                    // Serialization of owned strings cannot fail; keep the stream valid if it does.
                    Err(_) => return,
                }
            }
        };
        let _ = writeln!(self.stderr, "{line}");
    }

    /// Writes one record; text mode prefixes every rendered line with `indent`.
    pub(crate) fn record_indented(&mut self, record: &FileDiagnostic, indent: &str) {
        match self.format {
            DiagnosticFormat::Text => {
                let text = record.to_string().replace('\n', &format!("\n{indent}"));
                let _ = writeln!(self.stderr, "{indent}{text}");
            }
            DiagnosticFormat::Json => self.record(record),
        }
    }

    /// Forwards output already rendered in this reporter's format by another reporter.
    pub(crate) fn forward(&mut self, rendered: &[u8]) {
        let _ = self.stderr.write_all(rendered);
    }

    /// Writes every record of a failure and returns the failure exit code `1`.
    pub(crate) fn failure(&mut self, failure: &CliFailure) -> i32 {
        for record in failure.records() {
            self.record(record);
        }
        1
    }

    /// Writes all records in order.
    pub(crate) fn records(&mut self, records: &[FileDiagnostic]) {
        for record in records {
            self.record(record);
        }
    }

    /// Writes a progress or summary line in text mode only.
    pub(crate) fn progress(&mut self, line: impl fmt::Display) {
        if self.format == DiagnosticFormat::Text {
            let _ = writeln!(self.stderr, "{line}");
        }
    }

    /// Returns the raw text stream when the format is text, for multi-line progress output.
    pub(crate) fn text_stream(&mut self) -> Option<&mut dyn Write> {
        match self.format {
            DiagnosticFormat::Text => Some(&mut *self.stderr),
            DiagnosticFormat::Json => None,
        }
    }

    fn source(&mut self, path: &Path) -> Option<&str> {
        self.sources
            .entry(path.to_path_buf())
            .or_insert_with(|| std::fs::read_to_string(path).ok())
            .as_deref()
    }
}

#[cfg(test)]
mod tests {
    use fpas_diagnostics::codes::CLI_INPUT_UNSUPPORTED;

    use super::{DiagnosticFormat, Reporter};
    use crate::cli_output::CliFailure;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn requested_format_ignores_program_arguments() {
        assert_eq!(
            DiagnosticFormat::requested(&args(&["check", "--diagnostics", "json"])),
            DiagnosticFormat::Json
        );
        assert_eq!(
            DiagnosticFormat::requested(&args(&["run", "a.fpas", "--", "--diagnostics", "json"])),
            DiagnosticFormat::Text
        );
    }

    #[test]
    fn json_mode_writes_one_record_per_line_and_no_progress() {
        let mut output = Vec::new();
        let mut reporter = Reporter::new(DiagnosticFormat::Json, &mut output);
        reporter.progress("Checked 3 files");
        let failure =
            CliFailure::new(CLI_INPUT_UNSUPPORTED, "cannot run\nthis").with_help("Pass a program.");
        assert_eq!(reporter.failure(&failure), 1);

        let text = String::from_utf8(output).expect("UTF-8");
        let lines = text.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 1);
        let record: serde_json::Value = serde_json::from_str(lines[0]).expect("JSON record");
        assert_eq!(record["code"], "F5038");
        assert_eq!(record["message"], "cannot run\nthis");
        assert_eq!(record["hint"], "Pass a program.");
    }

    #[test]
    fn text_mode_keeps_progress_and_rendered_records() {
        let mut output = Vec::new();
        let mut reporter = Reporter::new(DiagnosticFormat::Text, &mut output);
        reporter.progress("Checked 3 files");
        reporter.failure(&CliFailure::new(CLI_INPUT_UNSUPPORTED, "cannot run"));
        assert_eq!(
            String::from_utf8(output).expect("UTF-8"),
            "Checked 3 files\nerror[F5038]: cannot run\n"
        );
    }
}
