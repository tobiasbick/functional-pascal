//! Native-only path requests shared by synchronous and suspended execution.

#[cfg(test)]
mod tests;

use super::{
    append, binary, directory, failure::Failure, glob, metadata, publication, read, results, temp,
    tree,
};
use crate::stdlib::intrinsic_args::{IntrinsicCall, expect_array, pop_string, pop_value};
use crate::stdlib::{AggregateFactory, StdError};
use crate::{FsIntrinsic as Op, Intrinsic, IntrinsicOwner, SourceLocation, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

enum Input {
    Empty,
    Path(String),
    Pair(String, String),
    Text(String, String),
    Bytes(String, Vec<u8>),
}

pub(crate) struct Call {
    operation: Op,
    input: Input,
}

pub(crate) enum Data {
    Done,
    Boolean(bool),
    Number(i64),
    Text(String),
    Bytes(Vec<u8>),
    Paths(Vec<String>),
    Temporary(Temporary),
}

/// Only unpublished temporary resources are removed. Caller-specified paths are never rolled back.
pub(crate) struct Temporary {
    path: Option<PathBuf>,
    directory: bool,
}

impl Temporary {
    fn publish(mut self) -> String {
        let Some(path) = self.path.take() else {
            unreachable!("temporary resource was already published")
        };
        path.to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/")
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            if self.directory {
                let _ = fs::remove_dir(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}

pub(crate) struct Outcome {
    operation: Op,
    resources: Vec<String>,
    outcome: Result<Data, Failure>,
}

pub(crate) fn supported(op: Op) -> bool {
    Intrinsic::Fs(op).owner() == IntrinsicOwner::Standard
}

pub(crate) fn infallible(op: Op) -> bool {
    matches!(
        op,
        Op::Exists | Op::IsFile | Op::IsDir | Op::IsSymLink | Op::TempDir
    )
}

impl Call {
    pub(crate) fn prepare_values(
        operation: Op,
        values: &[Value],
        factory: &dyn AggregateFactory,
        location: SourceLocation,
    ) -> Result<Result<Self, Failure>, StdError> {
        let mut call = IntrinsicCall::new(values, factory);
        let request = Self::prepare(operation, &mut call, location)?;
        let (consumed, _) = call.finish();
        if consumed != values.len() {
            return Err(crate::stdlib::error::std_runtime_error(
                fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                "Filesystem argument window has unexpected arguments",
                "Pass the documented filesystem arguments.",
                location,
            ));
        }
        Ok(request)
    }

    pub(crate) fn prepare(
        operation: Op,
        call: &mut IntrinsicCall<'_>,
        location: SourceLocation,
    ) -> Result<Result<Self, Failure>, StdError> {
        let input = match operation {
            Op::CurrentDir | Op::TempDir | Op::CreateTempDir | Op::CreateTempFile => Input::Empty,
            Op::Rename | Op::CopyFile | Op::CopyDirAll => {
                let target = pop_string(pop_value(call, location)?, location)?;
                let source = pop_string(pop_value(call, location)?, location)?;
                Input::Pair(source, target)
            }
            Op::WriteText | Op::WriteTextAtomic | Op::AppendText => {
                let text = pop_string(pop_value(call, location)?, location)?;
                let path = pop_string(pop_value(call, location)?, location)?;
                Input::Text(path, text)
            }
            Op::WriteBytes => {
                let values = expect_array(pop_value(call, location)?, location)?;
                let path = pop_string(pop_value(call, location)?, location)?;
                let bytes = match binary::validate_bytes(values, location)? {
                    Ok(bytes) => bytes,
                    Err(error) => return Ok(Err(error)),
                };
                Input::Bytes(path, bytes)
            }
            _ => Input::Path(pop_string(pop_value(call, location)?, location)?),
        };
        Ok(Ok(Self { operation, input }))
    }

    fn resources(&self) -> Vec<String> {
        match &self.input {
            Input::Empty => vec![],
            Input::Path(path) | Input::Text(path, _) | Input::Bytes(path, _) => vec![path.clone()],
            Input::Pair(source, target) => vec![source.clone(), target.clone()],
        }
    }

    pub(crate) fn run(&self, mut cancelled: impl FnMut() -> bool) -> Outcome {
        let outcome = if cancelled() {
            Err(Failure::Cancelled)
        } else {
            self.execute(&mut cancelled)
        };
        Outcome {
            operation: self.operation,
            resources: self.resources(),
            outcome,
        }
    }

    fn execute(&self, cancelled: &mut dyn FnMut() -> bool) -> Result<Data, Failure> {
        Ok(match (self.operation, &self.input) {
            (Op::ReadText, Input::Path(path)) => {
                Data::Text(read::read_text_controlled(path, cancelled)?)
            }
            (Op::ReadBytes, Input::Path(path)) => {
                Data::Bytes(binary::read_bytes_controlled(path, cancelled)?)
            }
            (Op::WriteBytes, Input::Bytes(path, bytes)) => {
                super::io::check(cancelled)?;
                let mut file = fs::File::create(path)?;
                super::io::write_all(&mut file, bytes, cancelled)?;
                Data::Done
            }
            (Op::WriteText, Input::Text(path, text)) => {
                super::io::check(cancelled)?;
                let mut file = fs::File::create(path)?;
                super::io::write_all(&mut file, text.as_bytes(), cancelled)?;
                Data::Done
            }
            (Op::WriteTextAtomic, Input::Text(path, text)) => {
                publication::write_text_atomic(Path::new(path), text, cancelled)?;
                Data::Done
            }
            (Op::AppendText, Input::Text(path, text)) => {
                append::append_text(Path::new(path), text, cancelled)?;
                Data::Done
            }
            (Op::DeleteFile, Input::Path(path)) => {
                fs::remove_file(path)?;
                Data::Done
            }
            (Op::DeleteDir, Input::Path(path)) => {
                fs::remove_dir(path)?;
                Data::Done
            }
            (Op::DeleteDirAll, Input::Path(path)) => {
                fs::remove_dir_all(path)?;
                Data::Done
            }
            (Op::Rename, Input::Pair(source, target)) => {
                fs::rename(source, target)?;
                Data::Done
            }
            (Op::CopyFile, Input::Pair(source, target)) => {
                fs::copy(source, target)?;
                Data::Done
            }
            (Op::CopyDirAll, Input::Pair(source, target)) => {
                tree::copy_dir_controlled(Path::new(source), Path::new(target), cancelled)?;
                Data::Done
            }
            (Op::ReadDir, Input::Path(path)) => {
                Data::Paths(directory::read_dir_paths(path, cancelled)?)
            }
            (Op::ReadDirAll, Input::Path(path)) => {
                Data::Paths(tree::read_dir_all_controlled(path, cancelled)?)
            }
            (Op::Glob, Input::Path(path)) => {
                Data::Paths(glob::glob_paths_controlled(path, cancelled)?)
            }
            (Op::Exists, Input::Path(path)) => Data::Boolean(Path::new(path).exists()),
            (Op::IsFile, Input::Path(path)) => Data::Boolean(Path::new(path).is_file()),
            (Op::IsDir, Input::Path(path)) => Data::Boolean(Path::new(path).is_dir()),
            (Op::IsSymLink, Input::Path(path)) => {
                Data::Boolean(metadata::is_symlink(Path::new(path)))
            }
            (Op::FileSize, Input::Path(path)) => {
                Data::Number(metadata::file_size(Path::new(path))?)
            }
            (Op::ModifiedTime, Input::Path(path)) => {
                Data::Number(metadata::modified_time_millis(Path::new(path))?)
            }
            (Op::CreateDir, Input::Path(path)) => {
                fs::create_dir(path)?;
                Data::Done
            }
            (Op::CreateDirAll, Input::Path(path)) => {
                fs::create_dir_all(path)?;
                Data::Done
            }
            (Op::SetCurrentDir, Input::Path(path)) => {
                std::env::set_current_dir(path)?;
                Data::Done
            }
            (Op::CurrentDir, Input::Empty) => Data::Text(
                std::env::current_dir()?
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/"),
            ),
            (Op::TempDir, Input::Empty) => Data::Text(temp::temp_dir()),
            (Op::CreateTempDir, Input::Empty) => Data::Temporary(Temporary {
                path: Some(temp::create_temp_dir_path(cancelled)?),
                directory: true,
            }),
            (Op::CreateTempFile, Input::Empty) => Data::Temporary(Temporary {
                path: Some(temp::create_temp_file_path(cancelled)?),
                directory: false,
            }),
            _ => unreachable!("validated path-operation input"),
        })
    }
}

impl Outcome {
    pub(crate) fn invalid(operation: Op, error: Failure) -> Self {
        Self {
            operation,
            resources: vec![],
            outcome: Err(error),
        }
    }

    pub(crate) fn into_value(
        self,
        factory: &dyn AggregateFactory,
        location: SourceLocation,
    ) -> Result<Value, StdError> {
        let convert = |data| match data {
            Data::Done => Value::Boolean(true),
            Data::Boolean(value) => Value::Boolean(value),
            Data::Number(value) => Value::Integer(value),
            Data::Text(text) => Value::Str(text.into()),
            Data::Temporary(temp) => Value::Str(temp.publish().into()),
            Data::Bytes(bytes) => Value::Array(
                bytes
                    .into_iter()
                    .map(|b| Value::Integer(i64::from(b)))
                    .collect(),
            ),
            Data::Paths(paths) => {
                Value::Array(paths.into_iter().map(|p| Value::Str(p.into())).collect())
            }
        };
        let operation = Intrinsic::Fs(self.operation).debugger_name();
        if infallible(self.operation) {
            return self.outcome.map(convert).map_err(|error| {
                crate::stdlib::error::std_runtime_error(
                    if matches!(error, Failure::Cancelled) {
                        fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
                    } else {
                        fpas_diagnostics::codes::RUNTIME_HOST_IO_FAILURE
                    },
                    format!("{operation}: {error}"),
                    "Retry after earlier native filesystem work has finished.",
                    location,
                )
            });
        }
        results::result(
            factory,
            &operation,
            &self.resources,
            self.outcome,
            convert,
            location,
        )
    }
}
