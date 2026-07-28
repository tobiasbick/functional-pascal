//! `Std.Fs` runtime implementation.
//!
//! Blocking filesystem operations safe to call from `go` tasks.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use crate::error::StdError;
use crate::intrinsic_args::{pop_string, pop_value};
use crate::limits::{MAX_GLOB_MATCHES, MAX_READ_TEXT_BYTES};
use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation, Value};
use glob::glob;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Execute a `Std.Fs` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Fs(FsIntrinsic::ReadText) => {
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_string(read_text_limited(&path)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteText) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_bool(fs::write(path, text)));
        }
        Intrinsic::Fs(FsIntrinsic::WriteTextAtomic) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            let path = pop_string(pop_value(stack, location)?, location)?;
            stack.push(result_bool(write_text_atomic(Path::new(&path), &text)));
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
            stack.push(result_string_array(glob_paths(&pattern)));
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

fn read_text_limited(path: &str) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    if let Ok(metadata) = file.metadata()
        && metadata.len() > MAX_READ_TEXT_BYTES
    {
        return Err(format!(
            "File `{path}` is larger than the maximum ReadText size of {MAX_READ_TEXT_BYTES} bytes.\n  help: Read smaller files, or split the input outside FPAS."
        ));
    }

    let mut limited = file.take(MAX_READ_TEXT_BYTES.saturating_add(1));
    let mut text = String::new();
    limited
        .read_to_string(&mut text)
        .map_err(|error| error.to_string())?;
    if text.len() as u64 > MAX_READ_TEXT_BYTES {
        return Err(format!(
            "File `{path}` exceeds the maximum ReadText size of {MAX_READ_TEXT_BYTES} bytes.\n  help: Read smaller files, or split the input outside FPAS."
        ));
    }
    Ok(text)
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

fn write_text_atomic(path: &Path, text: &str) -> io::Result<()> {
    let temporary = unique_sibling(path, ".tmp");
    let mut cleanup = FileCleanup::new(temporary.clone());
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    replace_file(path, &temporary)?;
    cleanup.disarm();
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(path: &Path, temporary: &Path) -> io::Result<()> {
    fs::rename(temporary, path)
}

#[cfg(windows)]
fn replace_file(path: &Path, temporary: &Path) -> io::Result<()> {
    let backup = unique_sibling(path, ".bak");
    let had_previous = path.exists();
    if had_previous {
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(temporary, path) {
        if had_previous {
            let _ = fs::rename(&backup, path);
        }
        return Err(error);
    }
    if had_previous {
        fs::remove_file(backup)?;
    }
    Ok(())
}

fn unique_sibling(path: &Path, suffix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let mut value = path.as_os_str().to_os_string();
    value.push(format!(".{}.{}{suffix}", std::process::id(), id));
    PathBuf::from(value)
}

struct FileCleanup {
    path: Option<PathBuf>,
}

impl FileCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

fn glob_paths(pattern: &str) -> Result<Vec<String>, String> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err(
            "Glob pattern must not be empty.\n  help: Pass a path or glob such as `src/**/*.fpas`."
                .to_string(),
        );
    }

    let path = Path::new(pattern);
    if !contains_glob_metacharacters(pattern) && path.is_file() {
        return Ok(vec![normalize_path_string(path)]);
    }

    if !contains_glob_metacharacters(pattern) {
        return Ok(Vec::new());
    }

    let pattern_text = path.to_string_lossy().replace('\\', "/");
    let mut matches = Vec::<String>::new();
    for entry in glob(&pattern_text).map_err(|error| {
        format!(
            "Invalid glob pattern `{pattern}`.\n  help: Use a valid glob such as `src/**/*.fpas`.\n  details: {error}"
        )
    })? {
        let entry = entry.map_err(|error| {
            format!("Error while evaluating glob pattern `{pattern}`.\n  details: {error}")
        })?;
        if entry.is_file() {
            if matches.len() >= MAX_GLOB_MATCHES {
                return Err(format!(
                    "Glob pattern `{pattern}` matched more than {MAX_GLOB_MATCHES} files.\n  help: Narrow the pattern so fewer files match."
                ));
            }
            matches.push(normalize_path_string(&entry));
        }
    }

    matches.sort();
    Ok(matches)
}

fn contains_glob_metacharacters(value: &str) -> bool {
    value.chars().any(|c| matches!(c, '*' | '?' | '[' | ']'))
}

fn normalize_path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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
        let mut stack = vec![Value::Str(path.clone().into())];
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
    fn write_text_atomic_creates_and_replaces_utf8_file_without_leftovers() {
        let path = unique_temp_path("atomic.txt");
        fs::write(&path, "old").expect("write old fixture");
        let mut stack = vec![
            Value::Str(path.clone().into()),
            Value::Str("replacement".into()),
        ];

        run_fs(FsIntrinsic::WriteTextAtomic, &mut stack);

        assert_eq!(stack, vec![Value::ResultOk(Box::new(Value::Boolean(true)))]);
        assert_eq!(fs::read_to_string(&path).expect("read"), "replacement");
        let parent = Path::new(&path).parent().expect("parent");
        let file_name = Path::new(&path)
            .file_name()
            .expect("file name")
            .to_string_lossy();
        let leftovers = fs::read_dir(parent)
            .expect("read parent")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(file_name.as_ref())
            })
            .count();
        assert_eq!(leftovers, 1);
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
        let dir_path = std::path::Path::new(&file_path)
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

    #[test]
    fn glob_returns_matching_files_in_sorted_order() {
        let dir = unique_temp_path("glob_dir");
        fs::create_dir_all(&dir).expect("create fixture dir");
        let files = [
            dir_path(&dir, "b.fpas"),
            dir_path(&dir, "a.fpas"),
            dir_path(&dir, "c.fpas"),
        ];
        for file in &files {
            fs::write(file, "").expect("write fixture");
        }

        let pattern = format!("{dir}/?.fpas");
        let mut stack = vec![Value::Str(pattern.into())];
        run_fs(FsIntrinsic::Glob, &mut stack);

        assert_eq!(
            stack,
            vec![Value::ResultOk(Box::new(Value::Array(
                vec![
                    Value::Str(normalize_path_string(Path::new(&files[1])).into()),
                    Value::Str(normalize_path_string(Path::new(&files[0])).into()),
                    Value::Str(normalize_path_string(Path::new(&files[2])).into()),
                ]
                .into()
            )))]
        );

        for file in &files {
            let _ = fs::remove_file(file);
        }
        let _ = fs::remove_dir(dir);
    }

    #[test]
    fn glob_expands_recursive_patterns() {
        let root = PathBuf::from(unique_temp_path("glob_recursive"));
        let nested = root.join("src").join("ui");
        fs::create_dir_all(&nested).expect("create nested dirs");
        let main = root.join("src").join("main.fpas");
        let menu = nested.join("menu.fpas");
        fs::write(&main, "").expect("write main");
        fs::write(&menu, "").expect("write menu");

        let pattern = format!(
            "{}/src/**/*.fpas",
            root.to_string_lossy().replace('\\', "/")
        );
        let mut stack = vec![Value::Str(pattern.into())];
        run_fs(FsIntrinsic::Glob, &mut stack);

        let Value::ResultOk(value) = &stack[0] else {
            panic!("expected Ok result, got {:?}", stack[0]);
        };
        let Value::Array(paths) = value.as_ref() else {
            panic!("expected array result, got {:?}", value);
        };
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[0],
            Value::Str(normalize_path_string(main.as_path()).into())
        );
        assert_eq!(
            paths[1],
            Value::Str(normalize_path_string(menu.as_path()).into())
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn glob_returns_empty_array_when_nothing_matches() {
        let mut stack = vec![Value::Str(
            format!(
                "{}/__FPAS_FS_GLOB_NO_MATCH__/*.fpas",
                unique_temp_path("glob_empty")
            )
            .into(),
        )];
        run_fs(FsIntrinsic::Glob, &mut stack);
        assert_eq!(
            stack,
            vec![Value::ResultOk(Box::new(Value::Array(vec![].into())))]
        );
    }

    #[test]
    fn glob_returns_error_for_invalid_pattern() {
        let mut stack = vec![Value::Str("[unclosed".into())];
        run_fs(FsIntrinsic::Glob, &mut stack);
        assert!(matches!(stack[0], Value::ResultError(_)));
    }

    #[test]
    fn glob_returns_single_existing_file_for_plain_path() {
        let path = unique_temp_path("single.fpas");
        fs::write(&path, "").expect("write fixture");

        let mut stack = vec![Value::Str(path.clone().into())];
        run_fs(FsIntrinsic::Glob, &mut stack);
        assert_eq!(
            stack,
            vec![Value::ResultOk(Box::new(Value::Array(
                vec![Value::Str(normalize_path_string(Path::new(&path)).into())].into()
            )))]
        );

        let _ = fs::remove_file(path);
    }

    fn dir_path(dir: &str, name: &str) -> String {
        Path::new(dir).join(name).to_string_lossy().into_owned()
    }
}
