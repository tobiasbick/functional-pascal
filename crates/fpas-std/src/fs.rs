//! `Std.Fs` runtime implementation.
//!
//! Blocking filesystem operations safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/fs.md` (from the repository root).

use crate::error::StdError;
use crate::helpers::{pop_string, pop_value};
use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation, Value};
use std::fs;
use std::io;
use std::path::Path;

/// Execute a `Std.Fs` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Fs(FsIntrinsic::ReadText) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_string(fs::read_to_string(path)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteText) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_bool(fs::write(path, text)));
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
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn result_string(result: io::Result<String>) -> Value {
    match result {
        Ok(value) => Value::ResultOk(Box::new(Value::Str(value))),
        Err(error) => Value::ResultError(Box::new(Value::Str(error.to_string()))),
    }
}

fn result_bool(result: io::Result<()>) -> Value {
    match result {
        Ok(()) => Value::ResultOk(Box::new(Value::Boolean(true))),
        Err(error) => Value::ResultError(Box::new(Value::Str(error.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn read_text_returns_file_contents() {
        let path = unique_temp_path("read.txt");
        fs::write(&path, "hello fs").expect("write fixture");
        let mut stack = vec![Value::Str(path.clone())];
        run_fs(FsIntrinsic::ReadText, &mut stack);
        assert_eq!(
            stack,
            vec![Value::ResultOk(Box::new(Value::Str("hello fs".into())))]
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn read_text_returns_error_for_missing_file() {
        let mut stack = vec![Value::Str("__FPAS_FS_MISSING_6A1C0F2E__.txt".into())];
        run_fs(FsIntrinsic::ReadText, &mut stack);
        assert!(matches!(stack[0], Value::ResultError(_)));
    }

    #[test]
    fn write_text_creates_utf8_file() {
        let path = unique_temp_path("output.txt");
        let mut stack = vec![Value::Str(path.clone()), Value::Str("written".into())];
        run_fs(FsIntrinsic::WriteText, &mut stack);
        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Boolean(true)))]);
        assert_eq!(fs::read_to_string(&path).expect("read"), "written");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn exists_reports_file_and_directory() {
        let file_path = unique_temp_path("note.txt");
        fs::write(&file_path, "x").expect("write");
        let dir_path = std::path::Path::new(&file_path)
            .parent()
            .expect("parent")
            .to_string_lossy()
            .into_owned();

        let mut stack = vec![Value::Str(file_path.clone())];
        run_fs(FsIntrinsic::Exists, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        stack = vec![Value::Str(dir_path)];
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
        let dir_path = std::path::Path::new(&file_path)
            .parent()
            .expect("parent")
            .to_string_lossy()
            .into_owned();

        let mut stack = vec![Value::Str(file_path.clone())];
        run_fs(FsIntrinsic::IsFile, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        stack = vec![Value::Str(file_path.clone())];
        run_fs(FsIntrinsic::IsDir, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(false)]);

        stack = vec![Value::Str(dir_path)];
        run_fs(FsIntrinsic::IsDir, &mut stack);
        assert_eq!(stack, vec![Value::Boolean(true)]);

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn create_dir_makes_directory() {
        let nested = unique_temp_path("nested");
        let mut stack = vec![Value::Str(nested.clone())];
        run_fs(FsIntrinsic::CreateDir, &mut stack);
        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Boolean(true)))]);
        assert!(Path::new(&nested).is_dir());
        let _ = fs::remove_dir(nested);
    }
}
