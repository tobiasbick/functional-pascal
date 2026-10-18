//! Staged source replacement with snapshot checks and recovery on publication failure.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use atomic_write_file::AtomicWriteFile;
use fpas_diagnostics::codes::{CLI_INPUT_UNSUPPORTED, CLI_OUTPUT_FAILED};
use fpas_language_service::DocumentSnapshot;

use super::Failure;
use crate::cli_output::CliFailure;

pub(super) struct Change {
    pub path: PathBuf,
    pub original: Arc<DocumentSnapshot>,
    pub replacement: String,
}

pub(super) struct Plan {
    pub inputs: Vec<Arc<DocumentSnapshot>>,
    pub changes: Vec<Change>,
}

impl Plan {
    pub(super) fn verify(&self) -> Result<(), Failure> {
        for snapshot in &self.inputs {
            let current =
                fs::read(snapshot.path()).map_err(|error| failure(snapshot.path(), error))?;
            if current != snapshot.source().as_bytes() {
                return Err(CliFailure::new(
                    CLI_INPUT_UNSUPPORTED,
                    format!(
                        "Source `{}` changed during rename; retry against the saved sources.",
                        snapshot.path().display()
                    ),
                )
                .into());
            }
        }
        Ok(())
    }

    pub(super) fn publish(self) -> Result<(), Failure> {
        self.verify()?;
        // Prepare every replacement before committing any source, so staging failures leave
        // the project untouched. AtomicWriteFile removes uncommitted temporary files on drop.
        let mut staged = Vec::with_capacity(self.changes.len());
        for change in &self.changes {
            let prepare = || -> std::io::Result<AtomicWriteFile> {
                let permissions = fs::metadata(&change.path)?.permissions();
                if permissions.readonly() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "source is read-only",
                    ));
                }
                let mut file = AtomicWriteFile::open(&change.path)?;
                file.write_all(change.replacement.as_bytes())?;
                file.as_file().set_permissions(permissions)?;
                Ok(file)
            };
            staged.push(prepare().map_err(|error| failure(&change.path, error))?);
        }
        self.verify()?;
        for (index, file) in staged.into_iter().enumerate() {
            let change = &self.changes[index];
            let current = fs::read(&change.path);
            let result = match current {
                Ok(current) if current == change.original.source().as_bytes() => file.commit(),
                Ok(_) => Err(std::io::Error::other("source changed before publication")),
                Err(error) => Err(error),
            };
            if let Err(error) = result {
                let mut message = format!(
                    "Cannot publish rename to `{}`: {error}",
                    change.path.display()
                );
                for committed in self.changes[..=index].iter().rev() {
                    let restore = || -> std::io::Result<()> {
                        let current = fs::read(&committed.path)?;
                        if current == committed.original.source().as_bytes() {
                            return Ok(());
                        }
                        if current != committed.replacement.as_bytes() {
                            return Err(std::io::Error::other(
                                "source changed after publication; retained the newer text",
                            ));
                        }
                        crate::source_publication::write_source(
                            &committed.path,
                            committed.original.source(),
                        )
                    };
                    if let Err(error) = restore() {
                        message.push_str(&format!(
                            "\nCannot restore `{}`: {error}",
                            committed.path.display()
                        ));
                    }
                }
                return Err(CliFailure::new(CLI_OUTPUT_FAILED, message).into());
            }
        }
        Ok(())
    }
}

pub(super) fn failure(path: &Path, error: impl std::fmt::Display) -> Failure {
    CliFailure::new(
        CLI_OUTPUT_FAILED,
        format!("Cannot prepare rename for `{}`: {error}", path.display()),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::create_temp_dir;
    use fpas_language_service::LanguageService;

    #[test]
    fn rename_refuses_changed_sources_including_unedited_analysis_inputs() {
        for changed_index in [0, 1] {
            let cwd = create_temp_dir("rename-publication-stale");
            let paths = [cwd.join("main.fpas"), cwd.join("other.fpas")];
            let original = "program T; begin discard 1; end.";
            let newer = "program T; begin discard 3; end.";
            for path in &paths {
                fs::write(path, original).unwrap();
            }
            let mut service = LanguageService::load(&paths[0]);
            let inputs = paths
                .iter()
                .map(|path| service.snapshot(path).unwrap())
                .collect::<Vec<_>>();
            let plan = Plan {
                changes: vec![Change {
                    path: paths[0].clone(),
                    original: Arc::clone(&inputs[0]),
                    replacement: "program T; begin discard 2; end.".into(),
                }],
                inputs,
            };
            fs::write(&paths[changed_index], newer).unwrap();
            let result = plan.publish();
            assert!(
                matches!(result, Err(Failure::Cli(ref failure)) if failure.to_string().contains("changed during rename"))
            );
            for (index, path) in paths.iter().enumerate() {
                assert_eq!(
                    fs::read_to_string(path).unwrap(),
                    if index == changed_index {
                        newer
                    } else {
                        original
                    }
                );
            }
        }
    }
}
