//! Cold and warm CLI project builds, including process startup and artifact admission.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use super::fixture_directory::FixtureDirectory;

/// Measures complete release CLI builds with fresh or reusable source-adjacent artifacts.
pub(super) fn run(iterations: usize, mode: &str) -> Result<(), String> {
    if !matches!(mode, "cold" | "warm") {
        return Err("Build mode must be cold or warm; example: cargo bench-fpas native project-build 3 warm".to_owned());
    }
    let root = std::env::current_dir().map_err(|error| error.to_string())?;
    let executable = match std::env::var_os("FPAS_BENCH_CLI") {
        Some(path) => path.into(),
        None => crate::suite::ensure_release_fpas(&root)?,
    };
    let mut elapsed = Duration::ZERO;
    for _ in 0..iterations {
        let fixture = FixtureDirectory::create(&root)?;
        let directory = fixture.path();
        copy_sources(&root.join("lib"), &directory.join("lib"))?;
        std::fs::copy(
            root.join("examples/pascal/tui/headless_render_benchmark.fpas"),
            directory.join("main.fpas"),
        )
        .map_err(|error| error.to_string())?;
        std::fs::write(
            directory.join("build.fpasprj"),
            "[project]\nname = \"build-benchmark\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"main.fpas\"]\n",
        )
        .map_err(|error| error.to_string())?;
        if mode == "warm" {
            build(&executable, directory, false)?;
        }
        let started = Instant::now();
        build(&executable, directory, mode == "warm")?;
        elapsed += started.elapsed();
    }
    println!(
        "builds: {iterations}\nmode: {mode}\nelapsed: {} ms",
        elapsed.as_millis()
    );
    Ok(())
}

fn build(executable: &Path, directory: &Path, expect_reuse: bool) -> Result<(), String> {
    let output = Command::new(executable)
        .arg("build")
        .arg("--std-lib")
        .arg(directory.join("lib"))
        .arg(directory.join("build.fpasprj"))
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "Build workload failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !directory.join("build-benchmark.fpascp").is_file() {
        return Err("Build workload did not publish its program image".to_owned());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if expect_reuse
        && !stdout
            .lines()
            .any(|line| line.starts_with("Reused program `build-benchmark`:"))
    {
        return Err(format!("Warm build must reuse its program image: {stdout}"));
    }
    Ok(())
}

fn copy_sources(source: &Path, destination: &Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if kind.is_dir() {
            copy_sources(&path, &target)?;
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "fpas" || extension == "fpasprj")
        {
            std::fs::copy(&path, &target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FixtureDirectory, copy_sources};
    use std::path::Path;

    #[test]
    fn cold_fixture_copies_nested_sources_and_manifests_without_artifacts() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixture = FixtureDirectory::create(&root).unwrap();
        let source = fixture.path().join("source");
        let destination = fixture.path().join("copy");
        std::fs::create_dir_all(source.join("nested")).unwrap();
        for name in [
            "library.fpasprj",
            "nested/unit.fpas",
            "nested/unit.fpascu",
            "program.fpascp",
        ] {
            std::fs::write(source.join(name), name).unwrap();
        }

        copy_sources(&source, &destination).unwrap();

        assert_eq!(
            std::fs::read_to_string(destination.join("nested/unit.fpas")).unwrap(),
            "nested/unit.fpas"
        );
        assert!(destination.join("library.fpasprj").is_file());
        assert!(!destination.join("nested/unit.fpascu").exists());
        assert!(!destination.join("program.fpascp").exists());
    }
}
