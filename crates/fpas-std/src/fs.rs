//! `Std.Fs` runtime implementation.
//!
//! Blocking filesystem operations safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

mod glob;
mod publication;
mod read;

use std::fs;
use std::io;
use std::path::Path;

use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation, Value};

use crate::error::StdError;
use crate::intrinsic_args::{pop_string, pop_value};

/// Execute a `Std.Fs` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Fs(FsIntrinsic::ReadText) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_string(read::read_text_limited(&path)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteText) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_bool(fs::write(path, text)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteTextAtomic) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_bool(publication::write_text_atomic(
                Path::new(&path),
                &text,
            )));
        }
        Intrinsic::Fs(FsIntrinsic::Exists) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(Path::new(&path).exists()));
        }
        Intrinsic::Fs(FsIntrinsic::IsFile) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(Path::new(&path).is_file()));
        }
        Intrinsic::Fs(FsIntrinsic::IsDir) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(Value::Boolean(Path::new(&path).is_dir()));
        }
        Intrinsic::Fs(FsIntrinsic::CreateDir) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_bool(fs::create_dir(path)));
        }
        Intrinsic::Fs(FsIntrinsic::Glob) => {
            let pattern = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_string_array(glob::glob_paths(&pattern)));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn result_string(result: Result<String, String>) -> Value {
    match result {
        Ok(value) => Value::ResultOk(Box::new(Value::Str(value.into()))),
        Err(error) => Value::ResultError(Box::new(Value::Str(error.into()))),
    }
}

fn result_bool(result: io::Result<()>) -> Value {
    match result {
        Ok(()) => Value::ResultOk(Box::new(Value::Boolean(true))),
        Err(error) => Value::ResultError(Box::new(Value::Str(error.to_string().into()))),
    }
}

fn result_string_array(result: Result<Vec<String>, String>) -> Value {
    match result {
        Ok(paths) => Value::ResultOk(Box::new(Value::Array(
            paths
                .into_iter()
                .map(|path| Value::Str(path.into()))
                .collect(),
        ))),
        Err(message) => Value::ResultError(Box::new(Value::Str(message.into()))),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_fs(intrinsic: FsIntrinsic, stack: &mut Vec<Value>) {
        run(Intrinsic::Fs(intrinsic), stack, test_location()).unwrap();
    }

    fn unique_temp_path(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("fpas_fs_test_{nanos}_{name}"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn write_text_creates_utf8_file() {
        let path = unique_temp_path("output.txt");
        let mut stack = vec![
            Value::Str(path.clone().into()),
            Value::Str("written".into()),
        ];

        run_fs(FsIntrinsic::WriteText, &mut stack);

        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Boolean(true)))]);
        assert_eq!(fs::read_to_string(&path).expect("read"), "written");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn exists_reports_file_and_directory() {
        let file_path = unique_temp_path("note.txt");
        fs::write(&file_path, "x").expect("write");
        let dir_path = Path::new(&file_path)
            .parent()
            .expect("parent")
            .to_string_lossy()
            .into_owned();

        let mut stack = vec![Value::Str(file_path.clone().into())];
        run_fs(FsIntrinsic::Exists, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        stack = vec![Value::Str(dir_path.into())];
        run_fs(FsIntrinsic::Exists, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        stack = vec![Value::Str("__FPAS_FS_MISSING_9D44B0A1__".into())];
        run_fs(FsIntrinsic::Exists, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(false)]);

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn is_file_and_is_dir_distinguish_entries() {
        let file_path = unique_temp_path("entry.txt");
        fs::write(&file_path, "x").expect("write");
        let dir_path = Path::new(&file_path)
            .parent()
            .expect("parent")
            .to_string_lossy()
            .into_owned();

        let mut stack = vec![Value::Str(file_path.clone().into())];
        run_fs(FsIntrinsic::IsFile, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        stack = vec![Value::Str(file_path.clone().into())];
        run_fs(FsIntrinsic::IsDir, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(false)]);

        stack = vec![Value::Str(dir_path.into())];
        run_fs(FsIntrinsic::IsDir, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn create_dir_makes_directory() {
        let nested = unique_temp_path("nested");
        let mut stack = vec![Value::Str(nested.clone().into())];

        run_fs(FsIntrinsic::CreateDir, &mut stack);

        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Boolean(true)))]);
        assert!(Path::new(&nested).is_dir());
        let _ = fs::remove_dir(nested);
    }
}
