//! `fpas build` project and workspace artifact orchestration.
//!
//! Documentation: `docs/pascal/program-structure/cli.md`,
//! `docs/pascal/tools/diagnostics.md`.

use std::io::Write;
use std::path::Path;

use fpas_diagnostics::codes::{
    CLI_ARGUMENTS_INVALID, CLI_INPUT_UNSUPPORTED, CLI_OUTPUT_FAILED, PROJECT_PATH_INVALID,
};
use fpas_project as project;

use crate::cli_input::{BuildCliConfig, CliInput};
use crate::cli_output::{CliFailure, Reporter};

/// Builds artifacts for one resolved project or workspace input.
pub(crate) fn build_cli(
    config: BuildCliConfig,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    if config.executable {
        return build_native_cli(config, standard_library, stdout, reporter);
    }
    match config.input {
        CliInput::ProjectFile(path) => {
            build_project_file(&path, standard_library, stdout, reporter)
        }
        CliInput::WorkspaceFile(path) => {
            build_workspace_file(&path, standard_library, stdout, reporter)
        }
        CliInput::SourceFile(path) => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!("Cannot build source input `{}`.", path.display()),
            )
            .with_help("Pass a `.fpasprj` or `.fpasworkspace` file."),
        ),
        CliInput::CompiledProgramFile(path) => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!("Cannot build compiled program input `{}`.", path.display()),
            )
            .with_help("Pass its `.fpasprj` or `.fpasworkspace` source manifest."),
        ),
    }
}

fn build_native_cli(
    config: BuildCliConfig,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    match config.input {
        CliInput::ProjectFile(path) => {
            let default_name = match project::load_project(&path) {
                Ok(loaded) => loaded.name,
                Err(error) => return reporter.failure(&error.into()),
            };
            let name = config.name.as_deref().unwrap_or(&default_name);
            build_native_project(
                &path,
                path.parent(),
                name,
                standard_library,
                stdout,
                reporter,
            )
        }
        CliInput::WorkspaceFile(path) => {
            let workspace = match project::load_workspace(&path) {
                Ok(workspace) => workspace,
                Err(error) => return reporter.failure(&error.into()),
            };
            let program = match project::discover_run_project_in_workspace(&path) {
                Ok(program) => program,
                Err(error) => return reporter.failure(&error.into()),
            };
            let name = config.name.as_deref().unwrap_or(&workspace.name);
            build_native_project(
                &program,
                path.parent(),
                name,
                standard_library,
                stdout,
                reporter,
            )
        }
        CliInput::SourceFile(path) | CliInput::CompiledProgramFile(path) => reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!(
                    "Cannot build a native application from `{}`.",
                    path.display()
                ),
            )
            .with_help(
                "Pass a program `.fpasprj` or a `.fpasworkspace` containing exactly one program.",
            ),
        ),
    }
}

fn build_native_project(
    project_path: &Path,
    output_directory: Option<&Path>,
    application_name: &str,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    if let Err(message) = crate::native_executable::validate_application_name(application_name) {
        return reporter.failure(&CliFailure::from_message(CLI_ARGUMENTS_INVALID, &message));
    }
    let Some(output_directory) = output_directory else {
        return reporter.failure(&CliFailure::new(
            PROJECT_PATH_INVALID,
            format!(
                "Cannot resolve native application output directory for `{}`.",
                project_path.display()
            ),
        ));
    };
    let loaded = match project::load_project(project_path) {
        Ok(loaded) => loaded,
        Err(error) => return reporter.failure(&error.into()),
    };
    if loaded.kind != project::ProjectKind::Program {
        return reporter.failure(
            &CliFailure::new(
                CLI_INPUT_UNSUPPORTED,
                format!(
                    "Native applications require a `program` project; `{}` is not executable.",
                    project_path.display()
                ),
            )
            .with_help("Pass a program `.fpasprj`."),
        );
    }
    reporter.records(&loaded.warnings);
    let artifact =
        match crate::project_build::build_program_artifact(project_path, &loaded, standard_library)
        {
            Ok(artifact) => artifact,
            Err(failure) => return reporter.failure(&failure),
        };
    match crate::native_executable::package(&artifact.path, output_directory, application_name) {
        Ok(output) => {
            let _ = writeln!(
                stdout,
                "Built application `{application_name}`: {}",
                output.display()
            );
            0
        }
        Err(message) => reporter.failure(&CliFailure::from_message(CLI_OUTPUT_FAILED, &message)),
    }
}

fn build_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(error) => return reporter.failure(&error.into()),
    };
    reporter.records(&loaded.warnings);

    let result = match loaded.kind {
        project::ProjectKind::Program => {
            crate::project_build::build_program_artifact(path, &loaded, standard_library).map(
                |artifact| {
                    let action = if artifact.reused { "Reused" } else { "Built" };
                    format!(
                        "{action} program `{}`: {}",
                        loaded.name,
                        artifact.path.display()
                    )
                },
            )
        }
        project::ProjectKind::Library => {
            crate::project_build::check_library(&loaded, standard_library)
                .map(|()| format!("Built library `{}`.", loaded.name))
        }
        project::ProjectKind::Test => {
            crate::project_build::check_test_project(&loaded, standard_library)
                .map(|()| format!("Built test project `{}`.", loaded.name))
        }
    };

    match result {
        Ok(message) => {
            let _ = writeln!(stdout, "{message}");
            0
        }
        Err(failure) => reporter.failure(&failure),
    }
}

fn build_workspace_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(error) => return reporter.failure(&error.into()),
    };

    let mut exit_code = 0;
    for member in &workspace.member_projects {
        let member_exit = build_project_file(member, standard_library, stdout, reporter);
        if member_exit != 0 {
            exit_code = member_exit;
        }
    }
    exit_code
}
