//! Validation for names used as CLI-produced artifact filenames.

/// Returns whether `name` is safe to use as one filesystem path component.
pub(crate) fn is_valid(name: &str) -> bool {
    !name.trim().is_empty()
        && !matches!(name, "." | "..")
        && !name.contains(['/', '\\', '\0'])
        && !windows_name_is_invalid(name)
}

fn windows_name_is_invalid(name: &str) -> bool {
    if !cfg!(windows) {
        return false;
    }
    if name.contains(['<', '>', ':', '"', '|', '?', '*']) || name.ends_with(['.', ' ']) {
        return true;
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
}

#[cfg(test)]
mod tests {
    use super::is_valid;

    #[test]
    fn rejects_names_that_are_not_one_non_empty_path_component() {
        for name in ["", " ", ".", "..", "dir/app", "dir\\app", "app\0name"] {
            assert!(!is_valid(name), "`{name}` must be rejected");
        }
    }

    #[cfg(windows)]
    #[test]
    fn rejects_windows_reserved_names_and_syntax() {
        for name in [
            "NUL",
            "con.txt",
            "PRN",
            "aux.data",
            "COM1",
            "lpt9.log",
            "app:debug",
            "app?debug",
            "app*debug",
            "app.",
            "app ",
        ] {
            assert!(!is_valid(name), "`{name}` must be rejected");
        }
    }

    #[test]
    fn accepts_portable_artifact_names() {
        for name in ["app", "my-app", "app.debug", "ConsoleApp"] {
            assert!(is_valid(name), "`{name}` must be accepted");
        }
    }
}
