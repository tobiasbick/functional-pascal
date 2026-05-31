use super::*;

#[test]
fn std_path_base_name_dir_name_and_extension() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Path;
begin
  WriteLn(BaseName('dir/nested/file.txt'));
  WriteLn(DirName('dir/nested/file.txt'));
  WriteLn(Extension('archive.tar.gz'))
end.",
    );
    assert_eq!(out.lines, vec!["file.txt", "dir/nested", "gz"]);
}

#[test]
fn std_path_join_and_normalize_use_final_component() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Path;
begin
  WriteLn(BaseName(Join(['one', 'two', 'file.txt'])));
  WriteLn(BaseName(Normalize('dir/nested/../file.txt')))
end.",
    );
    assert_eq!(out.lines, vec!["file.txt", "file.txt"]);
}

#[test]
fn std_path_qualified_calls_work() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Path;
begin
  WriteLn(Std.Path.Extension('notes.md'))
end.",
    );
    assert_eq!(out.lines, vec!["md"]);
}
