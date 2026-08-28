//! Host-runner discovery and native application packaging.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn package(
    program_image: &Path,
    output_directory: &Path,
    application_name: &str,
) -> Result<PathBuf, String> {
    if !cfg!(any(windows, target_os = "linux")) {
        return Err(
            "Native FPAS applications are currently supported only on Windows and Linux."
                .to_string(),
        );
    }
    validate_application_name(application_name)?;
    let runner_path = runner_path()?;
    let runner = fs::read(&runner_path).map_err(|error| {
        format!(
            "Cannot read native FPAS runner `{}`: {error}",
            runner_path.display()
        )
    })?;
    let image = fs::read(program_image).map_err(|error| {
        format!(
            "Cannot read compiled program `{}`: {error}",
            program_image.display()
        )
    })?;
    let bundled = fpas_bundle::encode(&runner, &image, application_name)
        .map_err(|error| format!("Cannot bundle application `{application_name}`: {error}"))?;
    let output = output_directory.join(native_filename(application_name));
    fpas_bundle::publish(&output, &bundled)?;
    Ok(output)
}

fn runner_path() -> Result<PathBuf, String> {
    let executable = env::current_exe()
        .map_err(|error| format!("Cannot locate the running `fpas` executable: {error}"))?;
    let directory = executable.parent().ok_or_else(|| {
        format!(
            "Cannot resolve the directory containing `{}`.",
            executable.display()
        )
    })?;
    let runner = directory.join(native_filename("fpas-runner"));
    if runner.is_file() {
        return Ok(runner);
    }
    Err(format!(
        "Native FPAS runner not found at `{}`.\n  help: Build or install `fpas-runner` beside the `fpas` executable.",
        runner.display()
    ))
}

pub(crate) fn validate_application_name(name: &str) -> Result<(), String> {
    if !crate::artifact_filename::is_valid(name) {
        return Err(format!(
            "Application name `{name}` cannot be used as an executable filename.\n  help: Use a non-empty name without path separators or Windows-reserved filename syntax."
        ));
    }
    Ok(())
}

fn native_filename(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}
