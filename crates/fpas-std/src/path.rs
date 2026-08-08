//! `Std.Path` runtime implementation.
//!
//! Pure path manipulation without filesystem access.
//!
//! **Documentation:** `docs/pascal/std/host/path.md` (from the repository root).

use crate::error::StdError;
use crate::intrinsic_args::{
    IntrinsicCall, pop_array, pop_string, pop_value, value_as_string_for_join,
};
use fpas_bytecode::{Intrinsic, PathIntrinsic, SourceLocation, Value};
use std::path::{Component, MAIN_SEPARATOR, Path, PathBuf};

/// Execute a `Std.Path` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Path(PathIntrinsic::Join) => {
            let segments = pop_array(pop_value(call, location)?, location)?;
            let mut buf = PathBuf::new();
            for value in segments {
                let segment = value_as_string_for_join(&value, location)?;
                buf.push(segment);
            }
            call.push(Value::Str(buf.to_string_lossy().into_owned().into()));
        }
        Intrinsic::Path(PathIntrinsic::BaseName) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(base_name(&path).into()));
        }
        Intrinsic::Path(PathIntrinsic::DirName) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(dir_name(&path).into()));
        }
        Intrinsic::Path(PathIntrinsic::Extension) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(extension(&path).into()));
        }
        Intrinsic::Path(PathIntrinsic::Normalize) => {
            let path = pop_string(pop_value(call, location)?, location)?;
            call.push(Value::Str(normalize_path(&path).into()));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn base_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn dir_name(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|ext| ext.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let parsed = Path::new(path);
    let prefix = parsed.components().find_map(|component| {
        if let Component::Prefix(prefix) = component {
            Some(prefix.as_os_str().to_os_string())
        } else {
            None
        }
    });
    let has_root = parsed.has_root();
    let mut parts: Vec<String> = Vec::new();

    for component in parsed.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => {
                if has_root {
                    if !parts.is_empty() {
                        parts.pop();
                    }
                } else if parts.last().is_some_and(|part| part != "..") {
                    parts.pop();
                } else {
                    parts.push("..".to_string());
                }
            }
            Component::Normal(name) => parts.push(name.to_string_lossy().into_owned()),
        }
    }

    let mut out = PathBuf::new();
    if let Some(prefix) = prefix {
        out.push(prefix);
    }
    if has_root && out.components().count() == 0 {
        out.push(MAIN_SEPARATOR.to_string());
    }
    for part in parts {
        out.push(part);
    }

    if out.as_os_str().is_empty() {
        if path == "." {
            return ".".to_string();
        }
        if path == ".." {
            return "..".to_string();
        }
    }

    out.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_path(intrinsic: PathIntrinsic, stack: &mut Vec<Value>) {
        crate::run_intrinsic(Intrinsic::Path(intrinsic), stack, test_location()).unwrap();
    }

    #[test]
    fn join_combines_segments_with_platform_separator() {
        let mut stack = vec![Value::Array(
            vec![
                Value::Str("a".into()),
                Value::Str("b".into()),
                Value::Str("file.txt".into()),
            ]
            .into(),
        )];
        run_path(PathIntrinsic::Join, &mut stack);
        let expected = PathBuf::from("a")
            .join("b")
            .join("file.txt")
            .to_string_lossy()
            .into_owned();
        assert_eq!(stack, vec![Value::Str(expected.into())]);
    }

    #[test]
    fn join_empty_array_returns_empty_string() {
        let mut stack = vec![Value::Array(vec![].into())];
        run_path(PathIntrinsic::Join, &mut stack);
        assert_eq!(stack, vec![Value::Str(String::new().into())]);
    }

    #[test]
    fn base_name_returns_final_component() {
        let mut stack = vec![Value::Str("dir/nested/file.txt".into())];
        run_path(PathIntrinsic::BaseName, &mut stack);
        assert_eq!(stack, vec![Value::Str("file.txt".into())]);
    }

    #[test]
    fn base_name_returns_empty_for_trailing_separator_on_unix() {
        #[cfg(unix)]
        {
            let mut stack = vec![Value::Str("dir/nested/".into())];
            run_path(PathIntrinsic::BaseName, &mut stack);
            assert_eq!(stack, vec![Value::Str((String::new()).into())]);
        }
    }

    #[test]
    fn base_name_returns_last_segment_for_trailing_separator_on_windows() {
        #[cfg(windows)]
        {
            let mut stack = vec![Value::Str("dir/nested/".into())];
            run_path(PathIntrinsic::BaseName, &mut stack);
            assert_eq!(stack, vec![Value::Str("nested".into())]);
        }
    }

    #[test]
    fn dir_name_returns_parent_path() {
        let mut stack = vec![Value::Str("dir/nested/file.txt".into())];
        run_path(PathIntrinsic::DirName, &mut stack);
        assert_eq!(stack, vec![Value::Str("dir/nested".into())]);
    }

    #[test]
    fn dir_name_returns_empty_for_rootless_file_name() {
        let mut stack = vec![Value::Str("file.txt".into())];
        run_path(PathIntrinsic::DirName, &mut stack);
        assert_eq!(stack, vec![Value::Str(String::new().into())]);
    }

    #[test]
    fn extension_returns_suffix_without_dot() {
        let mut stack = vec![Value::Str("archive.tar.gz".into())];
        run_path(PathIntrinsic::Extension, &mut stack);
        assert_eq!(stack, vec![Value::Str("gz".into())]);
    }

    #[test]
    fn extension_returns_empty_when_missing() {
        let mut stack = vec![Value::Str("README".into())];
        run_path(PathIntrinsic::Extension, &mut stack);
        assert_eq!(stack, vec![Value::Str(String::new().into())]);
    }

    #[test]
    fn normalize_collapses_dot_segments() {
        let mut stack = vec![Value::Str("a/./b".into())];
        run_path(PathIntrinsic::Normalize, &mut stack);
        let expected = PathBuf::from("a").join("b").to_string_lossy().into_owned();
        assert_eq!(stack, vec![Value::Str(expected.into())]);
    }

    #[test]
    fn normalize_resolves_parent_dir_segments() {
        let mut stack = vec![Value::Str("a/b/../c".into())];
        run_path(PathIntrinsic::Normalize, &mut stack);
        let expected = PathBuf::from("a").join("c").to_string_lossy().into_owned();
        assert_eq!(stack, vec![Value::Str(expected.into())]);
    }

    #[test]
    fn normalize_preserves_dot_path() {
        let mut stack = vec![Value::Str(".".into())];
        run_path(PathIntrinsic::Normalize, &mut stack);
        assert_eq!(stack, vec![Value::Str(".".into())]);
    }

    #[test]
    fn normalize_preserves_parent_only_path() {
        let mut stack = vec![Value::Str("../..".into())];
        run_path(PathIntrinsic::Normalize, &mut stack);
        let expected = PathBuf::from("..")
            .join("..")
            .to_string_lossy()
            .into_owned();
        assert_eq!(stack, vec![Value::Str(expected.into())]);
    }

    #[cfg(windows)]
    #[test]
    fn normalize_uses_backslashes_on_windows() {
        let mut stack = vec![Value::Str("a/b\\c".into())];
        run_path(PathIntrinsic::Normalize, &mut stack);
        assert_eq!(stack, vec![Value::Str("a\\b\\c".into())]);
    }

    #[cfg(unix)]
    #[test]
    fn join_absolute_mid_segment_replaces_prefix_on_unix() {
        let mut stack = vec![Value::Array(
            vec![Value::Str("home".into()), Value::Str("/etc/hosts".into())].into(),
        )];
        run_path(PathIntrinsic::Join, &mut stack);
        assert_eq!(stack, vec![Value::Str("/etc/hosts".into())]);
    }

    #[cfg(windows)]
    #[test]
    fn join_absolute_mid_segment_replaces_prefix_on_windows() {
        let mut stack = vec![Value::Array(
            vec![Value::Str("home".into()), Value::Str("C:\\Windows".into())].into(),
        )];
        run_path(PathIntrinsic::Join, &mut stack);
        assert_eq!(stack, vec![Value::Str("C:\\Windows".into())]);
    }
}
