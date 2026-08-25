use super::*;
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(super) struct SharedWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl SharedWriter {
    fn into_string(self) -> String {
        let bytes = self
            .buffer
            .lock()
            .expect("shared writer buffer lock must succeed")
            .clone();
        String::from_utf8(bytes).expect("shared writer output must be UTF-8")
    }
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer
            .lock()
            .expect("shared writer buffer lock must succeed")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) fn run_and_capture_stderr(path: &str, source: &str) -> (i32, String) {
    let mut stderr = Vec::<u8>::new();
    let exit_code = run_source(path, source, Box::new(std::io::sink()), &mut stderr);
    let stderr_output = String::from_utf8(stderr).expect("stderr must be valid UTF-8");
    (exit_code, stderr_output)
}

pub(super) fn run_cli_and_capture_output(project_file: &Path, cwd: &Path) -> (i32, String, String) {
    run_cli_args_and_capture_output(
        &[
            String::from("run"),
            project_file.to_string_lossy().to_string(),
        ],
        cwd,
    )
}

pub(super) fn run_cli_args_and_capture_output(
    args: &[String],
    cwd: &Path,
) -> (i32, String, String) {
    let stdout = SharedWriter::default();
    let mut stderr = Vec::<u8>::new();
    let exit_code = run_cli(args, cwd, Box::new(stdout.clone()), &mut stderr);

    let stdout_output = stdout.into_string();
    let stderr_output = String::from_utf8(stderr).expect("stderr must be valid UTF-8");
    (exit_code, stdout_output, stderr_output)
}

pub(super) fn run_program_with_graph_and_capture_output(
    path: &Path,
    program_graph: &fpas_project::ProgramUnitGraph,
) -> (i32, String, String) {
    let stdout = SharedWriter::default();
    let mut stderr = Vec::<u8>::new();
    let built = match crate::project_build::build_test_program_with_graph(path, program_graph) {
        Ok(built) => built,
        Err(message) => {
            let _ = writeln!(stderr, "{message}");
            return (
                1,
                stdout.into_string(),
                String::from_utf8(stderr).expect("stderr must be valid UTF-8"),
            );
        }
    };
    let mut vm = fpas_vm::Vm::with_writer(built.executable, Box::new(stdout.clone()));
    let exit_code = match vm.run() {
        Ok(_) => 0,
        Err(diagnostic) => {
            let path_text = path.to_string_lossy();
            let _ = writeln!(
                stderr,
                "{}",
                crate::cli_run::render_cli_diagnostic_with_sources(
                    path_text.as_ref(),
                    Some(&built.source_paths),
                    &diagnostic,
                )
            );
            2
        }
    };
    (
        exit_code,
        stdout.into_string(),
        String::from_utf8(stderr).expect("stderr must be valid UTF-8"),
    )
}

pub(super) fn run_source_and_capture_output(path: &str, source: &str) -> (i32, String, String) {
    let stdout = SharedWriter::default();
    let mut stderr = Vec::<u8>::new();
    let exit_code = run_source(path, source, Box::new(stdout.clone()), &mut stderr);
    let stdout_output = stdout.into_string();
    let stderr_output = String::from_utf8(stderr).expect("stderr must be valid UTF-8");
    (exit_code, stdout_output, stderr_output)
}
