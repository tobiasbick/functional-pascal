//! Machine-readable installed-toolchain discovery.

use std::io::Write;

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainEnvironment {
    schema_version: u32,
    version: &'static str,
    executable: std::path::PathBuf,
    standard_library: std::path::PathBuf,
}

pub(crate) fn write_environment(stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(
                stderr,
                "Cannot locate the running `fpas` executable: {error}"
            );
            return 1;
        }
    };
    let standard_library = match crate::standard_library::default_standard_library_root() {
        Ok(Some(path)) => path,
        Ok(None) => {
            let _ = writeln!(
                stderr,
                "Functional Pascal standard library was not found beside `{}`.\n  help: Install the toolchain with its adjacent `lib/stdlib.fpasprj` directory.",
                executable.display()
            );
            return 1;
        }
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return 1;
        }
    };
    let environment = ToolchainEnvironment {
        schema_version: 1,
        version: env!("CARGO_PKG_VERSION"),
        executable,
        standard_library,
    };
    match crate::cli_output::write_stdout(
        stdout,
        stderr,
        "toolchain environment to stdout",
        |out| {
            serde_json::to_writer(&mut *out, &environment)?;
            writeln!(out)
        },
    ) {
        Ok(()) => 0,
        Err(exit_code) => exit_code,
    }
}
