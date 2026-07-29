//! `fpas build` project and workspace artifact orchestration.
//!
//! Documentation: `docs/pascal/program-structure/cli.md`.

use std::io::Write;
use std::path::Path;

use fpas_project as project;

use crate::cli_input::{BuildCliConfig, CliInput};

/// Builds artifacts for one resolved project or workspace input.
pub(crate) fn build_cli(
    config: BuildCliConfig,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    if config.executable {
        return build_native_cli(config, standard_library, stdout, stderr);
    }
    match config.input {
        CliInput::ProjectFile(path) => build_project_file(&path, standard_library, stdout, stderr),
        CliInput::WorkspaceFile(path) => {
            build_workspace_file(&path, standard_library, stdout, stderr)
        }
        CliInput::SourceFile(path) => {
            let _ = writeln!(
                stderr,
                "Cannot build source input `{}`.\n  help: Pass a `.fpasprj` or `.fpasworkspace` file.",
                path.display()
            );
            1
        }
        CliInput::CompiledProgramFile(path) => {
            let _ = writeln!(
                stderr,
                "Cannot build compiled program input `{}`.\n  help: Pass its `.fpasprj` or `.fpasworkspace` source manifest.",
                path.display()
            );
            1
        }
    }
}

fn build_native_cli(
    config: BuildCliConfig,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    match config.input {
        CliInput::ProjectFile(path) => {
            let default_name = match project::load_project(&path) {
                Ok(loaded) => loaded.name,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            let name = config.name.as_deref().unwrap_or(&default_name);
            build_native_project(&path, path.parent(), name, standard_library, stdout, stderr)
        }
        CliInput::WorkspaceFile(path) => {
            let workspace = match project::load_workspace(&path) {
                Ok(workspace) => workspace,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            let program = match project::discover_run_project_in_workspace(&path) {
                Ok(program) => program,
                Err(message) => {
                    let _ = writeln!(stderr, "{message}");
                    return 1;
                }
            };
            let name = config.name.as_deref().unwrap_or(&workspace.name);
            build_native_project(
                &program,
                path.parent(),
                name,
                standard_library,
                stdout,
                stderr,
            )
        }
        CliInput::SourceFile(path) | CliInput::CompiledProgramFile(path) => {
            let _ = writeln!(
                stderr,
                "Cannot build a native application from `{}`.\n  help: Pass a program `.fpasprj` or a `.fpasworkspace` containing exactly one program.",
                path.display()
            );
            1
        }
    }
}

fn build_native_project(
    project_path: &Path,
    output_directory: Option<&Path>,
    application_name: &str,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    if let Err(message) = crate::native_executable::validate_application_name(application_name) {
        let _ = writeln!(stderr, "{message}");
        return 1;
    }
    let Some(output_directory) = output_directory else {
        let _ = writeln!(
            stderr,
            "Cannot resolve native application output directory for `{}`.",
            project_path.display()
        );
        return 1;
    };
    let loaded = match project::load_project(project_path) {
        Ok(loaded) => loaded,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };
    if loaded.kind != project::ProjectKind::Program {
        let _ = writeln!(
            stderr,
            "Native applications require a `program` project; `{}` is not executable.\n  help: Pass a program `.fpasprj`.",
            project_path.display()
        );
        return 1;
    }
    for warning in &loaded.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }
    let artifact =
        match crate::project_build::build_program_artifact(project_path, &loaded, standard_library)
        {
            Ok(artifact) => artifact,
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
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
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            1
        }
    }
}

fn build_project_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let loaded = match project::load_project(path) {
        Ok(loaded) => loaded,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    for warning in &loaded.warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }

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
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            1
        }
    }
}

fn build_workspace_file(
    path: &Path,
    standard_library: Option<&project::StandardLibrary>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let workspace = match project::load_workspace(path) {
        Ok(workspace) => workspace,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };

    let mut exit_code = 0;
    for member in &workspace.member_projects {
        let member_exit = build_project_file(member, standard_library, stdout, stderr);
        if member_exit != 0 {
            exit_code = member_exit;
        }
    }
    exit_code
}
