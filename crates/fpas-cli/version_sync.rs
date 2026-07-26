use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn validate_std_version(path: &Path, expected: &str) -> io::Result<()> {
    let source = fs::read_to_string(path)?;
    for constant in ["CompilerVersion", "LibraryVersion"] {
        let actual = constant_value(&source, constant).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Std.Version must declare string constant `{constant}`"),
            )
        })?;
        if actual != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Std.Version `{constant}` is `{actual}`, but the Cargo package version is `{expected}`"
                ),
            ));
        }
    }
    Ok(())
}

fn constant_value<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    source.lines().find_map(|line| {
        let declaration = line.trim().strip_prefix(name)?;
        let value = declaration
            .strip_prefix(": string")?
            .trim_start()
            .strip_prefix(":=")?
            .trim();
        value
            .strip_suffix(';')?
            .trim()
            .strip_prefix('\'')?
            .strip_suffix('\'')
    })
}

#[cfg(test)]
mod tests {
    use super::validate_std_version;
    use crate::test_support::{create_temp_dir, write_text};
    use std::fs;

    #[test]
    fn accepts_matching_compiler_and_library_versions() {
        let root = create_temp_dir("stdlib-version-match");
        let source = root.join("Version.fpas");
        write_text(
            &source,
            "const\n  CompilerVersion: string := '1.2.3';\n  LibraryVersion: string := '1.2.3';\n",
        );

        validate_std_version(&source, "1.2.3").expect("matching versions must be accepted");
        fs::remove_dir_all(root).expect("temp directory must be removed");
    }

    #[test]
    fn rejects_a_mismatching_version() {
        let root = create_temp_dir("stdlib-version-mismatch");
        let source = root.join("Version.fpas");
        write_text(
            &source,
            "const\n  CompilerVersion: string := '1.2.3';\n  LibraryVersion: string := '1.2.2';\n",
        );

        let error = validate_std_version(&source, "1.2.3")
            .expect_err("mismatching versions must be rejected");
        assert!(error.to_string().contains("LibraryVersion"));
        assert!(error.to_string().contains("1.2.2"));
        assert!(error.to_string().contains("1.2.3"));
        fs::remove_dir_all(root).expect("temp directory must be removed");
    }

    #[test]
    fn rejects_a_missing_required_version() {
        let root = create_temp_dir("stdlib-version-missing");
        let source = root.join("Version.fpas");
        write_text(&source, "const\n  CompilerVersion: string := '1.2.3';\n");

        let error =
            validate_std_version(&source, "1.2.3").expect_err("missing versions must be rejected");
        assert!(error.to_string().contains("LibraryVersion"));
        fs::remove_dir_all(root).expect("temp directory must be removed");
    }
}
