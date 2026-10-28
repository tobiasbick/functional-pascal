//! `Std.Io.Fs` runtime implementation.
//!
//! Blocking filesystem operations safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

mod append;
mod binary;
mod directory;
mod failure;
mod glob;
mod metadata;
mod publication;
mod read;
mod results;
mod temp;
mod tree;

use std::fs;
use std::path::Path;

use crate::{FsIntrinsic, Intrinsic, SourceLocation, Value};

use crate::stdlib::error::StdError;
use crate::stdlib::intrinsic_args::{IntrinsicCall, expect_array, pop_string, pop_value};

/// Read a bounded UTF-8 file through the same implementation as Std.Io.Fs.ReadText.
pub fn read_text_for_host(path: &str) -> std::io::Result<String> {
    read::read_text_limited(path).map_err(|error| match error {
        failure::Failure::Io(error) => error,
        failure::Failure::LimitExceeded(_, _) => {
            std::io::Error::from(std::io::ErrorKind::FileTooLarge)
        }
        _ => std::io::Error::from(std::io::ErrorKind::InvalidData),
    })
}
/// Write a UTF-8 file through the same implementation as Std.Io.Fs.WriteText.
pub fn write_text_for_host(path: &str, text: &str) -> std::io::Result<()> {
    fs::write(path, text)
}

/// Execute a `Std.Io.Fs` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Fs(FsIntrinsic::ReadBytes) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.ReadBytes",
                std::slice::from_ref(&path),
                binary::read_bytes(&path),
                |bytes| {
                    Value::Array(
                        bytes
                            .into_iter()
                            .map(|byte| Value::Integer(i64::from(byte)))
                            .collect(),
                    )
                },
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::WriteBytes) => {
            let bytes = expect_array(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            let outcome = binary::write_bytes(&path, bytes, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.WriteBytes",
                std::slice::from_ref(&path),
                outcome,
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::ReadText) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.ReadText",
                std::slice::from_ref(&path),
                read::read_text_limited(&path),
                |v| Value::Str(v.into()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::WriteText) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.WriteText",
                std::slice::from_ref(&path),
                write_text_for_host(&path, &text).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::WriteTextAtomic) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.WriteTextAtomic",
                std::slice::from_ref(&path),
                (publication::write_text_atomic(Path::new(&path), &text)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::DeleteFile) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.DeleteFile",
                std::slice::from_ref(&path),
                (fs::remove_file(&path)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::DeleteDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.DeleteDir",
                std::slice::from_ref(&path),
                (fs::remove_dir(&path)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::DeleteDirAll) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.DeleteDirAll",
                std::slice::from_ref(&path),
                (fs::remove_dir_all(&path)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::Rename) => {
            let target = pop_string(pop_value(call, location)?, location)?;
            let source = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.Rename",
                &[source.clone(), target.clone()],
                (fs::rename(&source, &target)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::CopyFile) => {
            let target = pop_string(pop_value(call, location)?, location)?;
            let source = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CopyFile",
                &[source.clone(), target.clone()],
                (fs::copy(&source, &target).map(|_| ())).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::AppendText) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.AppendText",
                std::slice::from_ref(&path),
                (append::append_text(Path::new(&path), &text)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::FileSize) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.FileSize",
                std::slice::from_ref(&path),
                metadata::file_size(Path::new(&path)),
                Value::Integer,
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::ModifiedTime) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.ModifiedTime",
                std::slice::from_ref(&path),
                metadata::modified_time_millis(Path::new(&path)),
                Value::Integer,
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::IsSymLink) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(metadata::is_symlink(Path::new(&path))));
        }
        Intrinsic::Fs(FsIntrinsic::CurrentDir) => {
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CurrentDir",
                &[],
                (std::env::current_dir().map(|dir| {
                    dir.to_string_lossy()
                        .replace(std::path::MAIN_SEPARATOR, "/")
                }))
                .map_err(Into::into),
                |v| Value::Str(v.into()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::SetCurrentDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.SetCurrentDir",
                std::slice::from_ref(&path),
                (std::env::set_current_dir(&path)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::TempDir) => {
            call.push(Value::Str(temp::temp_dir().into()));
        }
        Intrinsic::Fs(FsIntrinsic::CreateTempDir) => {
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CreateTempDir",
                &[],
                temp::create_temp_dir(),
                |v| Value::Str(v.into()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::CreateTempFile) => {
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CreateTempFile",
                &[],
                temp::create_temp_file(),
                |v| Value::Str(v.into()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::ReadDirAll) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.ReadDirAll",
                std::slice::from_ref(&path),
                tree::read_dir_all_paths(&path),
                |paths| Value::Array(paths.into_iter().map(|p| Value::Str(p.into())).collect()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::CopyDirAll) => {
            let target = pop_string(pop_value(call, location)?, location)?;
            let source = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CopyDirAll",
                &[source.clone(), target.clone()],
                tree::copy_dir_all(Path::new(&source), Path::new(&target)),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::Exists) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(Path::new(&path).exists()));
        }
        Intrinsic::Fs(FsIntrinsic::IsFile) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(Path::new(&path).is_file()));
        }
        Intrinsic::Fs(FsIntrinsic::IsDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(Path::new(&path).is_dir()));
        }
        Intrinsic::Fs(FsIntrinsic::CreateDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CreateDir",
                std::slice::from_ref(&path),
                (fs::create_dir(&path)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::CreateDirAll) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.CreateDirAll",
                std::slice::from_ref(&path),
                (fs::create_dir_all(&path)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::ReadDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.ReadDir",
                std::slice::from_ref(&path),
                directory::read_dir_paths(&path),
                |paths| Value::Array(paths.into_iter().map(|p| Value::Str(p.into())).collect()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::Glob) => {
            let pattern = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Io.Fs.Glob",
                std::slice::from_ref(&pattern),
                glob::glob_paths(&pattern),
                |paths| Value::Array(paths.into_iter().map(|p| Value::Str(p.into())).collect()),
                location,
            )?);
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

#[cfg(test)]
mod binary_tests;

#[cfg(test)]
mod tests;
