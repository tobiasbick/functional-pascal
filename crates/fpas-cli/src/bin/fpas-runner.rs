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

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

fn main() {
    let exit_code = run();
    if exit_code != 0 {
        process::exit(exit_code);
    }
}

fn run() -> i32 {
    let executable = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Cannot locate FPAS application executable: {error}");
            return 1;
        }
    };
    let bytes = match fs::read(&executable) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!(
                "Cannot read FPAS application `{}`: {error}",
                executable.display()
            );
            return 1;
        }
    };
    let bundled = match fpas_bundle::decode(&bytes) {
        Ok(bundled) => bundled,
        Err(error) => {
            eprintln!(
                "Cannot load FPAS application `{}`: {error}",
                executable.display()
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
    if let Err(diagnostic) = vm.run() {
        let path = source_paths
            .get(usize::try_from(diagnostic.span.source_id()).unwrap_or(usize::MAX))
            .map_or_else(|| name.to_string(), |path| path.display().to_string());
        eprintln!("{}", fpas_diagnostics::render(&path, &diagnostic));
        return 2;
    }
    0
}
