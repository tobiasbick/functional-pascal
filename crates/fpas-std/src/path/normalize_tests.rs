//! Root and prefix preservation during lexical path normalization.

use super::normalize_path;

#[cfg(windows)]
#[test]
fn normalize_preserves_absolute_windows_drive_roots() {
    for (input, expected) in [
        (r"D:\projects\demo", r"D:\projects\demo"),
        ("D:/projects/demo", r"D:\projects\demo"),
        (r"D:\", r"D:\"),
        ("D:/", r"D:\"),
        (r"D:\projects\..\demo", r"D:\demo"),
        (r"D:\..\..\demo", r"D:\demo"),
        (r"D:\demo\..", r"D:\"),
        (r"\\?\D:\projects\demo", r"\\?\D:\projects\demo"),
    ] {
        let actual = normalize_path(input);
        assert_eq!(actual, expected, "input: {input}");
        assert!(std::path::Path::new(&actual).is_absolute(), "{actual}");
        assert_eq!(
            normalize_path(&actual),
            actual,
            "normalization is idempotent"
        );
    }
}

#[cfg(windows)]
#[test]
fn normalize_preserves_drive_relative_and_root_relative_paths() {
    for (input, expected) in [
        ("D:", "D:"),
        (r"D:demo", r"D:demo"),
        (r"D:projects\..\demo", r"D:demo"),
        (r"D:..\demo", r"D:..\demo"),
        (r"\projects\demo", r"\projects\demo"),
    ] {
        let actual = normalize_path(input);
        assert_eq!(actual, expected, "input: {input}");
        assert!(!std::path::Path::new(&actual).is_absolute(), "{actual}");
    }
}

#[cfg(windows)]
#[test]
fn normalize_preserves_unc_share_roots() {
    for (input, expected) in [
        (r"\\server\share\projects\..\demo", r"\\server\share\demo"),
        (r"\\server\share\..\..\demo", r"\\server\share\demo"),
        (r"\\?\UNC\server\share\demo", r"\\?\UNC\server\share\demo"),
    ] {
        let actual = normalize_path(input);
        assert_eq!(actual, expected, "input: {input}");
        assert!(std::path::Path::new(&actual).is_absolute(), "{actual}");
    }
}

#[test]
fn normalize_preserves_platform_root_and_clamps_parent_traversal() {
    let root = std::path::MAIN_SEPARATOR.to_string();
    assert_eq!(normalize_path(&root), root);
    assert_eq!(normalize_path(&format!("{root}demo/../..")), root);
    assert_eq!(normalize_path(""), "");
}
