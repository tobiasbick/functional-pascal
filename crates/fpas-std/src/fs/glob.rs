//! Stable, bounded glob expansion for `Std.Fs.Glob`.

use std::path::Path;

use glob::glob;

use crate::limits::MAX_GLOB_MATCHES;

pub(super) fn glob_paths(pattern: &str) -> Result<Vec<String>, String> {
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
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{glob_paths, normalize_path_string};

    fn unique_temp_path(name: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("fpas-fs-glob-{}-{id}-{name}", std::process::id()))
    }

    #[test]
    fn glob_returns_matching_files_in_sorted_order() {
        let dir = unique_temp_path("sorted");
        fs::create_dir_all(&dir).expect("create fixture dir");
        let files = [dir.join("b.fpas"), dir.join("a.fpas"), dir.join("c.fpas")];
        for file in &files {
            fs::write(file, "").expect("write fixture");
        }

        let pattern = format!("{}/*.fpas", normalize_path_string(&dir));
        let paths = glob_paths(&pattern).expect("glob result");

        assert_eq!(
            paths,
            vec![
                normalize_path_string(&files[1]),
                normalize_path_string(&files[0]),
                normalize_path_string(&files[2]),
            ]
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn glob_expands_recursive_patterns() {
        let root = unique_temp_path("recursive");
        let nested = root.join("src").join("ui");
        fs::create_dir_all(&nested).expect("create nested dirs");
        let main = root.join("src").join("main.fpas");
        let menu = nested.join("menu.fpas");
        fs::write(&main, "").expect("write main");
        fs::write(&menu, "").expect("write menu");

        let pattern = format!("{}/src/**/*.fpas", normalize_path_string(&root));
        let paths = glob_paths(&pattern).expect("glob result");

        assert_eq!(
            paths,
            vec![normalize_path_string(&main), normalize_path_string(&menu)]
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn glob_returns_empty_array_when_nothing_matches() {
        let root = unique_temp_path("empty");
        let pattern = format!("{}/*.fpas", normalize_path_string(&root));

        let paths = glob_paths(&pattern).expect("valid empty glob");

        assert!(paths.is_empty());
    }

    #[test]
    fn glob_returns_error_for_invalid_pattern() {
        let result = glob_paths("[unclosed");

        assert!(result.is_err());
    }

    #[test]
    fn glob_returns_single_existing_file_for_plain_path() {
        let path = unique_temp_path("single.fpas");
        fs::write(&path, "").expect("write fixture");

        let paths = glob_paths(&path.to_string_lossy()).expect("plain path result");

        assert_eq!(paths, vec![normalize_path_string(Path::new(&path))]);
        fs::remove_file(path).ok();
    }
}
