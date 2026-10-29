use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::{FsIntrinsic, Value};

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn run_fs(intrinsic: FsIntrinsic, stack: &mut Vec<Value>) {
    crate::stdlib::execute_test_intrinsic(Intrinsic::Fs(intrinsic), stack, test_location())
        .unwrap();
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
    let _ = fs::remove_file(&path);
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
