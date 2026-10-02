#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Thin host-native runner for bundled Functional Pascal bytecode.
//!
//! `FPAS_DIAGNOSTICS=json` selects JSON Lines diagnostics on stderr without
//! consuming application arguments; see `docs/pascal/program-structure/cli.md`.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use fpas_diagnostics::codes::{BUILD_ARTIFACT_ENCODING_FAILED, BUILD_ARTIFACT_IO_FAILED};
use fpas_diagnostics::{Diagnostic, FileDiagnostic};

/// Environment variable that selects the runner's diagnostic format.
const DIAGNOSTICS_VARIABLE: &str = "FPAS_DIAGNOSTICS";

fn main() {
    let exit_code = run(json_requested(
        env::var(DIAGNOSTICS_VARIABLE).ok().as_deref(),
    ));
    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Only the exact value `json` selects JSON; any other value keeps text output.
fn json_requested(value: Option<&str>) -> bool {
    value == Some("json")
}

fn run(json: bool) -> i32 {
    let executable = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return startup_failure(
                json,
                format!("Cannot locate FPAS application executable: {error}"),
            );
        }
    };
    let bytes = match fs::read(&executable) {
        Ok(bytes) => bytes,
        Err(error) => {
            return startup_failure(
                json,
                format!(
                    "Cannot read FPAS application `{}`: {error}",
                    executable.display()
                ),
            );
        }
    };
    let bundled = match fpas_bundle::decode(&bytes) {
        Ok(bundled) => bundled,
        Err(error) => {
            report(
                json,
                &FileDiagnostic::new(
                    Diagnostic::error_without_source(
                        BUILD_ARTIFACT_ENCODING_FAILED,
                        format!(
                            "Cannot load FPAS application `{}`: {error}",
                            executable.display()
                        ),
                        Some("Rebuild the application with `fpas build --executable`.".into()),
                    ),
                    None,
                ),
            );
            return 1;
        }
    };
    let fpas_bundle::BundledProgram { name, image } = bundled;
    let source_paths = image
        .source_paths()
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let args = env::args().skip(1).collect();
    let mut vm =
        fpas_vm::Vm::with_writer_and_args(image.into_executable(), Box::new(io::stdout()), args);
    vm.allow_process_lifecycle();
    if json {
        vm.set_program_stderr(Arc::new(|text: &str| {
            if let Ok(line) = fpas_diagnostics::render_program_stderr_json(text) {
                write_stderr_line(&line);
            }
        }));
    }
    if let Err(diagnostic) = vm.run() {
        let path = diagnostic
            .span
            .and_then(|span| usize::try_from(span.source_id()).ok())
            .and_then(|index| source_paths.get(index))
            .cloned()
            .unwrap_or_else(|| PathBuf::from(name.to_string()));
        report(json, &FileDiagnostic::new(*diagnostic, Some(path)));
        return 2;
    }
    0
}

fn startup_failure(json: bool, message: String) -> i32 {
    report(
        json,
        &FileDiagnostic::new(
            Diagnostic::error_without_source(BUILD_ARTIFACT_IO_FAILED, message, None),
            None,
        ),
    );
    1
}

fn report(json: bool, record: &FileDiagnostic) {
    let line = if json {
        let path = record.path.as_deref().map(|path| path.to_string_lossy());
        match fpas_diagnostics::render_json(path.as_deref(), None, &record.diagnostic) {
            Ok(line) => line,
            Err(_) => return,
        }
    } else {
        record.to_string()
    };
    write_stderr_line(&line);
}

// One write per line keeps records whole when VM tasks also write program output.
fn write_stderr_line(line: &str) {
    let _ = io::stderr().write_all(format!("{line}\n").as_bytes());
}

#[cfg(test)]
mod tests {
    use super::json_requested;

    #[test]
    fn only_the_exact_json_value_selects_json() {
        assert!(json_requested(Some("json")));
        assert!(!json_requested(Some("JSON")));
        assert!(!json_requested(Some("text")));
        assert!(!json_requested(None));
    }
}
