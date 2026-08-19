//! Debug target preparation and external protocol transport.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use fpas_program::{Digest, ProgramIdentity, ProgramImage};

use crate::{CliInput, DebugCliConfig, DebugProtocol};

#[derive(Debug)]
struct PreparedExecutable {
    executable: fpas_bytecode::VerifiedExecutable,
    sources: Vec<fpas_debug::DebugSourceContent>,
}

pub(crate) fn debug_cli(
    config: DebugCliConfig,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let library =
        match crate::standard_library::resolve_standard_library(config.standard_library.as_deref())
        {
            Ok(library) => library,
            Err(message) => return fail(stderr, message),
        };
    let target = match prepare_target(&config, library.as_ref()) {
        Ok(target) => target,
        Err(message) => return fail(stderr, message),
    };
    let limits = fpas_vm::DebugExecutionLimits {
        max_instructions: config.instruction_limit,
        timeout: config.timeout,
        max_output_bytes: config.output_limit,
        ..fpas_vm::DebugExecutionLimits::default()
    };
    let target = target.with_execution_limits(limits);
    let result = match config.protocol {
        DebugProtocol::Jsonl => {
            let server = match fpas_debug::jsonl::JsonlServer::new(target) {
                Ok(server) => server,
                Err(error) => return fail(stderr, format!("Cannot start debugger: {error}")),
            };
            match config.commands {
                Some(path) => File::open(&path)
                    .map_err(|error| {
                        io::Error::new(
                            error.kind(),
                            format!("cannot read command script `{}`: {error}", path.display()),
                        )
                    })
                    .and_then(|reader| fpas_debug::jsonl::serve_script(reader, stdout, server)),
                None => fpas_debug::jsonl::serve(io::stdin(), stdout, server),
            }
        }
        DebugProtocol::Dap => {
            if config.commands.is_some() {
                return fail(
                    stderr,
                    "`--commands` is only supported with `--protocol jsonl`.\n  help: DAP uses framed messages over stdin/stdout.",
                );
            }
            let server = match fpas_debug::dap::DapServer::new(target) {
                Ok(server) => server,
                Err(error) => return fail(stderr, format!("Cannot start debugger: {error}")),
            };
            fpas_debug::dap::serve(io::stdin(), stdout, server)
        }
    };
    match result {
        Ok(()) => 0,
        Err(error) => fail(stderr, format!("Debugger transport failed: {error}")),
    }
}

fn prepare_target(
    config: &DebugCliConfig,
    standard_library: Option<&fpas_project::StandardLibrary>,
) -> Result<fpas_debug::PreparedDebugTarget, String> {
    let prepared = prepare_executable(config, standard_library)?;
    let reload_config = config.clone();
    let reload_library = standard_library.cloned();
    Ok(
        fpas_debug::PreparedDebugTarget::new(prepared.executable, config.program_args.clone())
            .with_sources(prepared.sources)
            .with_reloader(move || {
                let prepared = prepare_executable(&reload_config, reload_library.as_ref())
                    .map_err(|_| reload_build_error())?;
                Ok(fpas_debug::ReloadedDebugTarget::new(prepared.executable)
                    .with_sources(prepared.sources))
            }),
    )
}

fn prepare_executable(
    config: &DebugCliConfig,
    standard_library: Option<&fpas_project::StandardLibrary>,
) -> Result<PreparedExecutable, String> {
    let prepared = match &config.input {
        CliInput::SourceFile(path) if path.is_dir() => {
            return Err(format!(
                "Cannot debug directory `{}`.\n  help: Pass a program source, project, workspace, or compiled image.",
                path.display()
            ));
        }
        CliInput::SourceFile(path) => prepare_source(path, &config.cwd, standard_library)?,
        CliInput::ProjectFile(path) => prepare_project(path, standard_library)?,
        CliInput::WorkspaceFile(path) => {
            let project = fpas_project::discover_run_project_in_workspace(path)?;
            prepare_project(&project, standard_library)?
        }
        CliInput::CompiledProgramFile(path) => prepare_image(path, config.source_root.as_deref())?,
    };
    Ok(prepared)
}

fn reload_build_error() -> fpas_vm::DebugSessionError {
    fpas_vm::DebugSessionError {
        kind: fpas_vm::DebugErrorKind::LiveImageBuildFailed,
        message: "the debugger launch target could not be rebuilt for live reload".to_string(),
        hint: "Fix source or project diagnostics with `fpas check`, then retry reload. Host paths and compiler output are not copied onto the debugger protocol."
            .to_string(),
    }
}

fn prepare_source(
    path: &Path,
    cwd: &Path,
    standard_library: Option<&fpas_project::StandardLibrary>,
) -> Result<PreparedExecutable, String> {
    if standard_library.is_some() {
        let built = crate::project_build::build_test_program(
            path,
            &[],
            &fpas_project::ProjectLinkMeta::default(),
            standard_library,
        )?;
        return install_debug_sources(built.executable, &built.source_paths, Some(cwd));
    }
    let source = fs::read_to_string(path)
        .map_err(|error| format!("Error reading `{}`: {error}", path.display()))?;
    let (program, diagnostics) = fpas_parser::parse(&source);
    if let Some(diagnostic) = diagnostics
        .iter()
        .map(fpas_parser::ParseDiagnostic::as_diagnostic)
        .find(|diagnostic| diagnostic.is_error())
    {
        return Err(fpas_diagnostics::render(
            &path.to_string_lossy(),
            diagnostic,
        ));
    }
    let executable = fpas_compiler::compile(&program).map_err(|diagnostics| {
        diagnostics
            .iter()
            .map(|diagnostic| fpas_diagnostics::render(&path.to_string_lossy(), diagnostic))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    install_debug_sources(executable, &[path.to_path_buf()], Some(cwd))
}

fn prepare_project(
    path: &Path,
    standard_library: Option<&fpas_project::StandardLibrary>,
) -> Result<PreparedExecutable, String> {
    let loaded = fpas_project::load_project(path)?;
    if loaded.kind != fpas_project::ProjectKind::Program {
        return Err(format!(
            "Project `{}` is not an executable program.\n  help: Debug a project with `kind = \"program\"`.",
            path.display()
        ));
    }
    let built = crate::project_build::build_program(&loaded, standard_library)?;
    install_debug_sources(built.executable, &built.source_paths, path.parent())
}

fn install_debug_sources(
    executable: fpas_bytecode::VerifiedExecutable,
    source_paths: &[PathBuf],
    root: Option<&Path>,
) -> Result<PreparedExecutable, String> {
    let root = root.unwrap_or_else(|| Path::new("."));
    let portable = source_paths
        .iter()
        .enumerate()
        .map(|(index, path)| portable_path(path, root, index))
        .collect::<Vec<_>>();
    let bytes = source_paths
        .iter()
        .map(|path| {
            fs::read(path)
                .map_err(|error| format!("Cannot read debug source `{}`: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let hashes = bytes.iter().map(Digest::of).collect::<Vec<_>>();
    let sources = portable
        .iter()
        .zip(source_paths)
        .zip(&bytes)
        .map(|((path, original_path), bytes)| {
            String::from_utf8(bytes.clone())
                .map(|content| fpas_debug::DebugSourceContent {
                    path: path.clone(),
                    original_path: Some(
                        original_path
                            .canonicalize()
                            .unwrap_or_else(|_| original_path.clone()),
                    ),
                    content,
                })
                .map_err(|error| format!("Debug source `{path}` is not valid UTF-8: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let source_hash = hashes.first().copied().unwrap_or_else(|| Digest::of([]));
    let executable = ProgramImage::new(
        ProgramIdentity {
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash,
            options_hash: Digest::of(b"debug-session"),
            units: Vec::new(),
        },
        portable,
        hashes,
        executable,
    )
    .map(ProgramImage::into_executable)
    .map_err(|error| format!("Cannot install debugger source identities: {error}"))?;
    Ok(PreparedExecutable {
        executable,
        sources,
    })
}

fn portable_path(path: &Path, root: &Path, index: usize) -> String {
    path.strip_prefix(root)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .map_or_else(
            || {
                format!(
                    "sources/{index}/{}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            },
            |relative| relative.to_string_lossy().replace('\\', "/"),
        )
}

fn prepare_image(path: &Path, source_root: Option<&Path>) -> Result<PreparedExecutable, String> {
    let source_root = source_root.ok_or_else(|| format!("Compiled debug target `{}` requires `--source-root <dir>`.\n  help: Point it at the root used to build the image.", path.display()))?;
    let root = source_root.canonicalize().map_err(|error| {
        format!(
            "Cannot resolve source root `{}`: {error}",
            source_root.display()
        )
    })?;
    let image =
        fpas_program::decode(&fs::read(path).map_err(|error| {
            format!("Cannot read compiled program `{}`: {error}", path.display())
        })?)
        .map_err(|error| format!("Cannot load compiled program `{}`: {error}", path.display()))?;
    let mut sources = Vec::new();
    for (portable, expected) in image.source_paths().iter().zip(image.source_hashes()) {
        let candidate = root.join(portable).canonicalize().map_err(|error| {
            format!(
                "Cannot resolve debug source `{portable}` below `{}`: {error}",
                root.display()
            )
        })?;
        if !candidate.starts_with(&root) {
            return Err(format!(
                "Debug source `{portable}` escapes source root `{}`.\n  help: Rebuild the image with portable source paths.",
                root.display()
            ));
        }
        let bytes = fs::read(&candidate).map_err(|error| {
            format!(
                "Cannot read debug source `{}`: {error}",
                candidate.display()
            )
        })?;
        let actual = Digest::of(&bytes);
        if actual != *expected {
            return Err(format!(
                "Debug source `{portable}` is stale for compiled program `{}`.\n  help: Rebuild the `.fpascp` or restore the exact source content.",
                path.display()
            ));
        }
        let content = String::from_utf8(bytes)
            .map_err(|error| format!("Debug source `{portable}` is not valid UTF-8: {error}"))?;
        sources.push(fpas_debug::DebugSourceContent {
            path: portable.clone(),
            original_path: Some(candidate),
            content,
        });
    }
    Ok(PreparedExecutable {
        executable: image.into_executable(),
        sources,
    })
}

fn fail(stderr: &mut dyn Write, message: impl std::fmt::Display) -> i32 {
    let _ = writeln!(stderr, "{message}");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_image_rejects_stale_and_escaping_sources() {
        let root = crate::test_support::create_temp_dir("debug-stale-image");
        let source_path = root.join("main.fpas");
        crate::test_support::write_text(&source_path, "program Main; begin end.\n");
        let source = fs::read_to_string(&source_path).expect("read source");
        let (program, diagnostics) = fpas_parser::parse(&source);
        assert!(diagnostics.is_empty());
        let executable = fpas_compiler::compile(&program).expect("compile image fixture");
        let image = ProgramImage::new(
            ProgramIdentity {
                compiler_version: "test".into(),
                bytecode_version: fpas_bytecode::BYTECODE_VERSION,
                source_hash: Digest::of(source.as_bytes()),
                options_hash: Digest::of([]),
                units: Vec::new(),
            },
            vec!["main.fpas".into()],
            vec![Digest::of(source.as_bytes())],
            executable,
        )
        .expect("construct image");
        let image_path = root.join("main.fpascp");
        fs::write(
            &image_path,
            fpas_program::encode(&image).expect("encode image"),
        )
        .expect("write image");
        let prepared = prepare_image(&image_path, Some(&root)).expect("matching source accepted");
        assert_eq!(prepared.sources[0].path, "main.fpas");
        assert_eq!(
            prepared.sources[0].original_path.as_deref(),
            Some(
                source_path
                    .canonicalize()
                    .expect("canonical source")
                    .as_path()
            )
        );
        crate::test_support::write_text(&source_path, "program Main; begin var X := 1 end.\n");
        let error = prepare_image(&image_path, Some(&root)).expect_err("stale source rejected");
        assert!(error.contains("is stale"));
        fs::remove_dir_all(root).expect("remove image temp directory");
    }
}
