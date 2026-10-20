//! `Std.Fs` runtime implementation.
//!
//! Blocking filesystem operations safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

mod append;
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

use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation, Value};

use crate::error::StdError;
use crate::intrinsic_args::{IntrinsicCall, pop_string, pop_value};

/// Execute a `Std.Fs` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Fs(FsIntrinsic::ReadText) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Fs.ReadText",
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
                "Std.Fs.WriteText",
                std::slice::from_ref(&path),
                (fs::write(&path, text)).map_err(Into::into),
                |()| Value::Boolean(true),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::WriteTextAtomic) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(results::result(
                call.factory(),
                "Std.Fs.WriteTextAtomic",
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
                "Std.Fs.DeleteFile",
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
                "Std.Fs.DeleteDir",
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
                "Std.Fs.DeleteDirAll",
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
                "Std.Fs.Rename",
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
                "Std.Fs.CopyFile",
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
                "Std.Fs.AppendText",
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
                "Std.Fs.FileSize",
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
                "Std.Fs.ModifiedTime",
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
                "Std.Fs.CurrentDir",
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
                "Std.Fs.SetCurrentDir",
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
                "Std.Fs.CreateTempDir",
                &[],
                temp::create_temp_dir(),
                |v| Value::Str(v.into()),
                location,
            )?);
        }
        Intrinsic::Fs(FsIntrinsic::CreateTempFile) => {
            call.push(results::result(
                call.factory(),
                "Std.Fs.CreateTempFile",
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
                "Std.Fs.ReadDirAll",
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
                "Std.Fs.CopyDirAll",
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
                "Std.Fs.CreateDir",
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
                "Std.Fs.CreateDirAll",
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
                "Std.Fs.ReadDir",
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
                "Std.Fs.Glob",
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
mod tests;
