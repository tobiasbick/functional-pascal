//! Host path contracts, partial results and lexical reconstruction.

use super::*;
use presentation::{change_extension, relative_to};

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn host(path: &str) -> String {
    path.replace('/', std::path::MAIN_SEPARATOR_STR)
}

#[test]
fn relative_paths_have_explicit_target_base_order_and_normalize_components() {
    for (target, base, expected) in [
        ("src/main.fpas", "src", "main.fpas"),
        ("tests/spec.fpas", "src", "../tests/spec.fpas"),
        ("src", "src/main.fpas", ".."),
        ("same/./path", "same/path", "."),
        ("a/..", "", "."),
        ("", "", "."),
        (".", "a", ".."),
        ("a/b", "", "a/b"),
        ("", "a/b", "../.."),
        ("../file", "dir", "../../file"),
        ("../file", "../dir", "../file"),
        ("../../file", "../dir", "../../file"),
        ("../a", "../a", "."),
        ("..", "..", "."),
        ("α/δ", "α/β", "../δ"),
        ("FILE", "file", "../FILE"),
    ] {
        assert_eq!(
            relative_to(target, base),
            Some(host(expected)),
            "target={target:?}, base={base:?}"
        );
    }
    for (target, base) in [("file", "../dir"), ("../file", "../../dir")] {
        assert_eq!(relative_to(target, base), None);
    }
}

#[test]
fn successful_relative_paths_reconstruct_the_normalized_target() {
    let paths = [
        "",
        ".",
        "..",
        "../..",
        "a",
        "a/b",
        "a/c",
        "a/..",
        "a/../b",
        "a/./b",
        "../a",
        "../b",
        "../../a",
        "a/../../b",
        "../a/../b",
        "α/β",
    ];
    for target in paths {
        for base in paths {
            if let Some(relative) = relative_to(target, base) {
                let joined = Path::new(base).join(relative);
                let joined = normalize_path(&joined.to_string_lossy());
                let target_normal = normalize_path(target);
                assert_eq!(
                    if joined.is_empty() { "." } else { &joined },
                    if target_normal.is_empty() {
                        "."
                    } else {
                        &target_normal
                    },
                    "target={target:?}, base={base:?}"
                );
            }
        }
    }
}

#[test]
fn extension_changes_only_the_final_suffix_and_preserves_parent_components() {
    for (path, extension, expected) in [
        ("archive.tar.gz", "zip", "archive.tar.zip"),
        ("archive.tar.gz", "", "archive.tar"),
        ("README", "txt", "README.txt"),
        ("README", "", "README"),
        (".env", "txt", ".env.txt"),
        (".env", "", ".env"),
        ("name.", "txt", "name.txt"),
        ("name.", "", "name"),
        ("folder/", "txt", "folder.txt"),
        ("dir/.", "txt", "dir.txt"),
        ("dir/../file.old", "new", "dir/../file.new"),
        ("file.old", "tar.gz", "file.tar.gz"),
        ("α/β.old", "新", "α/β.新"),
    ] {
        // Preserve input separators: ChangeExtension does not normalize the path.
        assert_eq!(
            change_extension(&host(path), extension),
            Some(host(expected)),
            "path={path:?}, extension={extension:?}"
        );
    }
}

#[test]
fn extension_change_returns_none_for_missing_names_and_invalid_extensions() {
    for path in ["", ".", "..", "dir/..", "/"] {
        assert_eq!(change_extension(path, "txt"), None, "path={path:?}");
    }
    for extension in [".txt", ".", "..", "/txt", "dir/txt", "a\0b"] {
        assert_eq!(
            change_extension("file.old", extension),
            None,
            "extension={extension:?}"
        );
    }
}

#[test]
fn path_intrinsics_use_boolean_and_option_values_without_reversing_arguments() {
    let location = test_location();
    let mut stack = vec![Value::Str("relative/file".into())];
    crate::execute_test_intrinsic(
        Intrinsic::Path(PathIntrinsic::IsAbsolute),
        &mut stack,
        location,
    )
    .unwrap();
    assert_eq!(stack, vec![Value::Boolean(false)]);
    let mut stack = vec![Value::Str("dir/file".into()), Value::Str("dir".into())];
    crate::execute_test_intrinsic(
        Intrinsic::Path(PathIntrinsic::RelativeTo),
        &mut stack,
        location,
    )
    .unwrap();
    assert_eq!(stack, vec![Value::option_some(Value::Str("file".into()))]);
    let mut stack = vec![Value::Str("file.old".into()), Value::Str("new".into())];
    crate::execute_test_intrinsic(
        Intrinsic::Path(PathIntrinsic::ChangeExtension),
        &mut stack,
        location,
    )
    .unwrap();
    assert_eq!(
        stack,
        vec![Value::option_some(Value::Str("file.new".into()))]
    );
    let mut stack = vec![Value::Str("file".into()), Value::Str("../dir".into())];
    crate::execute_test_intrinsic(
        Intrinsic::Path(PathIntrinsic::RelativeTo),
        &mut stack,
        location,
    )
    .unwrap();
    assert_eq!(stack, vec![Value::OptionNone]);
}

#[cfg(unix)]
#[test]
fn unix_roots_are_absolute_and_windows_spelling_remains_plain_text() {
    for (path, expected) in [
        ("/", true),
        ("/a", true),
        ("//a", true),
        ("", false),
        (".", false),
        (r"C:\a", false),
        (r"\a", false),
    ] {
        let mut stack = vec![Value::Str(path.into())];
        crate::execute_test_intrinsic(
            Intrinsic::Path(PathIntrinsic::IsAbsolute),
            &mut stack,
            test_location(),
        )
        .unwrap();
        assert_eq!(stack, vec![Value::Boolean(expected)], "{path}");
    }
    assert_eq!(relative_to("/a/file", "/b"), Some("../a/file".into()));
    assert_eq!(relative_to("/../../a", "/b/.."), Some("a".into()));
    assert_eq!(relative_to("/a", "a"), None);
    assert_eq!(relative_to("a", "/a"), None);
    assert_eq!(
        change_extension("file.old", r"a\b"),
        Some(r"file.a\b".into())
    );
}

#[cfg(windows)]
#[test]
fn windows_roots_drives_and_unc_shares_are_compared_without_ambient_state() {
    for (path, expected) in [
        (r"C:\a", true),
        (r"\\server\share\a", true),
        (r"C:a", false),
        (r"\a", false),
        ("/a", false),
        ("", false),
    ] {
        let mut stack = vec![Value::Str(path.into())];
        crate::execute_test_intrinsic(
            Intrinsic::Path(PathIntrinsic::IsAbsolute),
            &mut stack,
            test_location(),
        )
        .unwrap();
        assert_eq!(stack, vec![Value::Boolean(expected)], "{path}");
    }
    for (target, base, expected) in [
        (r"C:\a\file", r"C:\b", r"..\a\file"),
        (r"C:\a", r"C:\a", "."),
        (r"C:\a\..", r"C:\", "."),
        (r"\\server\share\a", r"\\server\share\b", r"..\a"),
        (r"\\?\C:\a", r"\\?\C:\b", r"..\a"),
        (r"dir\C:stream", "dir", r".\C:stream"),
        (r"dir\nested\C:stream", "dir", r"nested\C:stream"),
        (r"C:\dir\D:stream", r"C:\dir", r".\D:stream"),
        (r"C:\dir\nested\D:stream", r"C:\dir", r"nested\D:stream"),
    ] {
        assert_eq!(relative_to(target, base), Some(expected.into()));
    }
    for (target, base) in [
        (r"C:\a", r"D:\a"),
        (r"\\server\one\a", r"\\server\two\a"),
        (r"\a", r"\a"),
        (r"C:a", r"C:a"),
        ("C:", "C:"),
        (r"C:\a", "a"),
        ("a", r"C:\a"),
    ] {
        assert_eq!(relative_to(target, base), None);
    }
    assert_eq!(change_extension("file.old", r"a\b"), None);
    assert_eq!(change_extension(r"C:\", "txt"), None);
}
