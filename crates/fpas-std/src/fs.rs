//! `Std.Fs` runtime implementation.
//!
//! Blocking filesystem operations safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

mod append;
mod directory;
mod glob;
mod metadata;
mod publication;
mod read;
mod temp;
mod tree;

use std::fs;
use std::io;
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
            call.push(result_string(read::read_text_limited(&path)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteText) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::write(path, text)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteTextAtomic) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(publication::write_text_atomic(
                Path::new(&path),
                &text,
            )));
        }
        Intrinsic::Fs(FsIntrinsic::DeleteFile) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::remove_file(path)));
        }
        Intrinsic::Fs(FsIntrinsic::DeleteDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::remove_dir(path)));
        }
        Intrinsic::Fs(FsIntrinsic::DeleteDirAll) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::remove_dir_all(path)));
        }
        Intrinsic::Fs(FsIntrinsic::Rename) => {
            let target = pop_string(pop_value(call, location)?, location)?;
            let source = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::rename(source, target)));
        }
        Intrinsic::Fs(FsIntrinsic::CopyFile) => {
            let target = pop_string(pop_value(call, location)?, location)?;
            let source = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::copy(source, target).map(|_| ())));
        }
        Intrinsic::Fs(FsIntrinsic::AppendText) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(append::append_text(Path::new(&path), &text)));
        }
        Intrinsic::Fs(FsIntrinsic::FileSize) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_integer(metadata::file_size(Path::new(&path))));
        }
        Intrinsic::Fs(FsIntrinsic::ModifiedTime) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_integer(metadata::modified_time_millis(Path::new(
                &path,
            ))));
        }
        Intrinsic::Fs(FsIntrinsic::IsSymLink) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Boolean(metadata::is_symlink(Path::new(&path))));
        }
        Intrinsic::Fs(FsIntrinsic::CurrentDir) => {
            call.push(result_string(
                std::env::current_dir()
                    .map(|dir| {
                        dir.to_string_lossy()
                            .replace(std::path::MAIN_SEPARATOR, "/")
                    })
                    .map_err(|error| format!("CurrentDir failed: {error}")),
            ));
        }
        Intrinsic::Fs(FsIntrinsic::SetCurrentDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(std::env::set_current_dir(path)));
        }
        Intrinsic::Fs(FsIntrinsic::TempDir) => {
            call.push(Value::Str(temp::temp_dir().into()));
        }
        Intrinsic::Fs(FsIntrinsic::CreateTempDir) => {
            call.push(result_string(temp::create_temp_dir()));
        }
        Intrinsic::Fs(FsIntrinsic::CreateTempFile) => {
            call.push(result_string(temp::create_temp_file()));
        }
        Intrinsic::Fs(FsIntrinsic::ReadDirAll) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_string_array(tree::read_dir_all_paths(&path)));
        }
        Intrinsic::Fs(FsIntrinsic::CopyDirAll) => {
            let target = pop_string(pop_value(call, location)?, location)?;
            let source = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(
                tree::copy_dir_all(Path::new(&source), Path::new(&target))
                    .map_err(io::Error::other),
            ));
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
            call.push(result_bool(fs::create_dir(path)));
        }
        Intrinsic::Fs(FsIntrinsic::CreateDirAll) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_bool(fs::create_dir_all(path)));
        }
        Intrinsic::Fs(FsIntrinsic::ReadDir) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(result_string_array(directory::read_dir_paths(&path)));
        }
        Intrinsic::Fs(FsIntrinsic::Glob) => {
            let pattern = pop_string(pop_value(call, location)?, location)?;
            call.push(result_string_array(glob::glob_paths(&pattern)));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn result_string(result: Result<String, String>) -> Value {
    match result {
        Ok(value) => Value::result_ok(Value::Str(value.into())),
        Err(error) => Value::result_error(Value::Str(error.into())),
    }
}

fn result_integer(result: Result<i64, String>) -> Value {
    match result {
        Ok(value) => Value::result_ok(Value::Integer(value)),
        Err(error) => Value::result_error(Value::Str(error.into())),
    }
}

fn result_bool(result: io::Result<()>) -> Value {
    match result {
        Ok(()) => Value::result_ok(Value::Boolean(true)),
        Err(error) => Value::result_error(Value::Str(error.to_string().into())),
    }
}

fn result_string_array(result: Result<Vec<String>, String>) -> Value {
    match result {
        Ok(paths) => Value::result_ok(Value::Array(
            paths
                .into_iter()
                .map(|path| Value::Str(path.into()))
                .collect(),
        )),
        Err(message) => Value::result_error(Value::Str(message.into())),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_fs(intrinsic: FsIntrinsic, stack: &mut Vec<Value>) {
        crate::execute_test_intrinsic(Intrinsic::Fs(intrinsic), stack, test_location()).unwrap();
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

        assert_eq!(stack, vec![Value::result_ok(Value::Boolean(true))]);
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

        assert_eq!(stack, vec![Value::result_ok(Value::Boolean(true))]);
        assert!(Path::new(&nested).is_dir());
        let _ = fs::remove_dir(nested);
    }

    #[test]
    fn create_dir_all_creates_parents_and_accepts_existing_directory() {
        let root = unique_temp_path("create-all");
        let leaf = Path::new(&root).join("parent").join("leaf");
        let leaf_text = leaf.to_string_lossy().into_owned();

        for _ in 0..2 {
            let mut stack = vec![Value::Str(leaf_text.clone().into())];
            run_fs(FsIntrinsic::CreateDirAll, &mut stack);
            assert_eq!(stack, vec![Value::result_ok(Value::Boolean(true))]);
        }
        assert!(leaf.is_dir());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn create_dir_all_rejects_file_in_path() {
        let file_path = unique_temp_path("create-all-file");
        fs::write(&file_path, "not a directory").expect("fixture file must be written");
        let below_file = Path::new(&file_path)
            .join("child")
            .to_string_lossy()
            .into_owned();
        let mut stack = vec![Value::Str(below_file.into())];

        run_fs(FsIntrinsic::CreateDirAll, &mut stack);

        assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
        let _ = fs::remove_file(file_path);
    }

    fn run_path(intrinsic: FsIntrinsic, path: &Path) -> Value {
        let mut stack = vec![Value::Str(path.to_string_lossy().into_owned().into())];
        run_fs(intrinsic, &mut stack);
        stack.pop().expect("result")
    }

    fn run_pair(intrinsic: FsIntrinsic, first: &Path, second: &str) -> Value {
        let mut stack = vec![
            Value::Str(first.to_string_lossy().into_owned().into()),
            Value::Str(second.into()),
        ];
        run_fs(intrinsic, &mut stack);
        stack.pop().expect("result")
    }

    #[test]
    fn delete_dir_requires_empty_directory_and_delete_dir_all_recurses() {
        let root = PathBuf::from(unique_temp_path("delete-dir"));
        fs::create_dir_all(root.join("child")).expect("fixture directory");
        fs::write(root.join("child").join("file.txt"), "x").expect("fixture file");

        assert!(matches!(
            run_path(FsIntrinsic::DeleteDir, &root),
            Value::ResultError(_)
        ));
        assert_eq!(
            run_path(FsIntrinsic::DeleteDirAll, &root),
            Value::result_ok(Value::Boolean(true))
        );
        assert!(!root.exists());
        assert!(matches!(
            run_path(FsIntrinsic::DeleteDirAll, &root),
            Value::ResultError(_)
        ));

        fs::create_dir(&root).expect("empty directory");
        assert_eq!(
            run_path(FsIntrinsic::DeleteDir, &root),
            Value::result_ok(Value::Boolean(true))
        );
        assert!(!root.exists());
    }

    #[test]
    fn copy_rename_and_append_operate_on_files() {
        let root = PathBuf::from(unique_temp_path("copy-rename"));
        fs::create_dir_all(&root).expect("fixture directory");
        let source = root.join("a.txt");
        let copy = root.join("b.txt");
        let moved = root.join("c.txt");
        let ok = Value::result_ok(Value::Boolean(true));

        assert_eq!(run_pair(FsIntrinsic::AppendText, &source, "one"), ok);
        assert_eq!(run_pair(FsIntrinsic::AppendText, &source, "two"), ok);
        assert_eq!(fs::read_to_string(&source).expect("read"), "onetwo");

        assert_eq!(
            run_pair(FsIntrinsic::CopyFile, &source, &copy.to_string_lossy()),
            ok
        );
        assert_eq!(fs::read_to_string(&copy).expect("read"), "onetwo");

        assert_eq!(
            run_pair(FsIntrinsic::Rename, &copy, &moved.to_string_lossy()),
            ok
        );
        assert!(!copy.exists());
        assert_eq!(fs::read_to_string(&moved).expect("read"), "onetwo");

        assert!(matches!(
            run_pair(FsIntrinsic::CopyFile, &copy, &moved.to_string_lossy()),
            Value::ResultError(_)
        ));
        let _ = fs::remove_dir_all(root);
    }
}
