//! Select the executable and arguments for each bounded workload process.

use std::path::Path;
use std::process::Command;

use super::{BenchDriver, BenchSpec};

/// Builds a workload command without starting its process.
pub(super) fn command(repo_root: &Path, fpas: &Path, spec: &BenchSpec) -> Result<Command, String> {
    let mut command = match spec.driver {
        BenchDriver::Fpas => {
            let program = repo_root.join(&spec.path);
            if !program.is_file() {
                return Err(format!("benchmark source missing: {}", program.display()));
            }
            let mut command = Command::new(fpas);
            command.arg("run").arg(program).arg("--");
            command
        }
        BenchDriver::LanguageService
        | BenchDriver::LanguageServiceProject
        | BenchDriver::ProjectBuild
        | BenchDriver::CompilerLowering => {
            let executable = std::env::current_exe().map_err(|error| error.to_string())?;
            let mut command = Command::new(executable);
            let driver = match spec.driver {
                BenchDriver::CompilerLowering => "compiler-lowering",
                BenchDriver::LanguageServiceProject => "language-service-project",
                BenchDriver::ProjectBuild => "project-build",
                _ => "language-service",
            };
            command.args(["native", driver]).env("FPAS_BENCH_CLI", fpas);
            command
        }
    };
    command.args(&spec.args).current_dir(repo_root);
    Ok(command)
}
