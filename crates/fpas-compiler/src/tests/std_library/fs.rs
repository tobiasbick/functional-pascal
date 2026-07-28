use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn escape_pascal_string(value: &str) -> String {
    value.replace('\\', "\\\\")
}

fn unique_temp_path(name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("fpas_fs_pascal_{nanos}_{name}"))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn std_fs_reads_and_writes_utf8_text() {
    let path = unique_temp_path("roundtrip.txt");
    let escaped = escape_pascal_string(&path);
    let source = format!(
        "\
program T;
uses Std.Console, Std.Fs, Std.Result;
begin
  WriteLn(Std.Result.IsOk(WriteText('{escaped}', 'hello fs')));
  WriteLn(Std.Result.Unwrap(ReadText('{escaped}')))
end."
    );
    let out = compile_and_run(&source);
    assert_eq!(out.lines, vec!["true", "hello fs"]);
    let _ = fs::remove_file(path);
}

#[test]
fn std_fs_atomically_replaces_utf8_text() {
    let path = unique_temp_path("atomic.txt");
    fs::write(&path, "old").expect("write");
    let escaped = escape_pascal_string(&path);
    let source = format!(
        "\
program T;
uses Std.Console, Std.Fs, Std.Result;
begin
  WriteLn(Std.Result.IsOk(WriteTextAtomic('{escaped}', 'new')));
  WriteLn(Std.Result.Unwrap(ReadText('{escaped}')))
end."
    );

    let out = compile_and_run(&source);

    assert_eq!(out.lines, vec!["true", "new"]);
    let _ = fs::remove_file(path);
}

#[test]
fn std_fs_exists_is_file_and_is_dir() {
    let path = unique_temp_path("entry.txt");
    fs::write(&path, "x").expect("write");
    let dir = std::path::Path::new(&path)
        .parent()
        .expect("parent")
        .to_string_lossy()
        .into_owned();
    let escaped_file = escape_pascal_string(&path);
    let escaped_dir = escape_pascal_string(&dir);
    let source = format!(
        "\
program T;
uses Std.Console, Std.Fs;
begin
  WriteLn(Exists('{escaped_file}'));
  WriteLn(IsFile('{escaped_file}'));
  WriteLn(IsDir('{escaped_file}'));
  WriteLn(IsDir('{escaped_dir}'))
end."
    );
    let out = compile_and_run(&source);
    assert_eq!(out.lines, vec!["true", "true", "false", "true"]);
    let _ = fs::remove_file(path);
}

#[test]
fn std_fs_create_dir_and_read_text_with_go() {
    let dir = unique_temp_path("nested");
    let file = std::path::Path::new(&dir)
        .join("note.txt")
        .to_string_lossy()
        .into_owned();
    let escaped_dir = escape_pascal_string(&dir);
    let escaped_file = escape_pascal_string(&file);
    let source = format!(
        "\
program T;
uses Std.Console, Std.Fs, Std.Result, Std.Task;
begin
  WriteLn(Std.Result.IsOk(CreateDir('{escaped_dir}')));
  WriteLn(Std.Result.IsOk(WriteText('{escaped_file}', 'from task')));
  var ReadJob: task := go ReadText('{escaped_file}');
  WriteLn(Std.Result.Unwrap(Std.Task.Wait(ReadJob)))
end."
    );
    let out = compile_and_run(&source);
    assert_eq!(out.lines, vec!["true", "true", "from task"]);
    let _ = fs::remove_file(&file);
    let _ = fs::remove_dir(dir);
}

#[test]
fn std_fs_read_text_reports_missing_file_as_error() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Fs, Std.Result;
begin
  WriteLn(Std.Result.IsError(ReadText('__FPAS_FS_MISSING_2F8B91C4__.txt')))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}
