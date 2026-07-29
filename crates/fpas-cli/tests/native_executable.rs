#![expect(
    clippy::expect_used,
    reason = "native executable integration fixtures require direct filesystem and process assertions"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let path = std::env::temp_dir().join(format!(
        "fpas-native-application-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).expect("temporary directory must be created");
    path
}

fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory must be created");
    }
    fs::write(path, text).expect("fixture must be written");
}

#[test]
fn built_native_application_runs_without_project_files() {
    let root = temp_dir();
    let project = root.join("app.fpasprj");
    write(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &root.join("src/main.fpas"),
        "program Main;\nuses Std.Console, Std.Args;\nbegin\n  WriteLn(ParamStr(0))\nend.\n",
    );

    let fpas = env!("CARGO_BIN_EXE_fpas");
    let first = Command::new(fpas)
        .current_dir(&root)
        .args(["build", "--executable", "--name", "hello", "app.fpasprj"])
        .output()
        .expect("fpas build must start");
    let second = Command::new(fpas)
        .current_dir(&root)
        .args(["build", "--executable", "--name", "hello", "app.fpasprj"])
        .output()
        .expect("repeated fpas build must start");
    let executable = root.join(if cfg!(windows) { "hello.exe" } else { "hello" });

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(executable.is_file());
    let executable_bytes = fs::read(&executable).expect("application must be readable");
    if cfg!(windows) {
        assert!(executable_bytes.starts_with(b"MZ"));
    } else {
        assert!(executable_bytes.starts_with(b"\x7fELF"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        assert_ne!(
            fs::metadata(&executable)
                .expect("application metadata must be readable")
                .permissions()
                .mode()
                & 0o111,
            0
        );
    }

    fs::remove_file(&project).expect("manifest must be removable");
    fs::remove_dir_all(root.join("src")).expect("sources must be removable");
    fs::remove_file(root.join("app.fpascp")).expect("program image must be removable");
    let output = Command::new(&executable)
        .current_dir(&root)
        .arg("native argument")
        .output()
        .expect("bundled application must start");
    fs::remove_dir_all(&root).expect("temporary directory must be removed");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "native argument\n");
}

#[test]
fn workspace_application_defaults_to_workspace_name_and_location() {
    let root = temp_dir();
    write(
        &root.join("suite.fpasworkspace"),
        r#"[workspace]
name = "suite"
members = ["apps/hello/hello.fpasprj"]
"#,
    );
    write(
        &root.join("apps/hello/hello.fpasprj"),
        r#"[project]
name = "hello"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &root.join("apps/hello/src/main.fpas"),
        "program Main;\nuses Std.Console;\nbegin\n  WriteLn('workspace bundle')\nend.\n",
    );

    let build = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["build", "--executable", "suite.fpasworkspace"])
        .output()
        .expect("workspace build must start");
    let executable = root.join(if cfg!(windows) { "suite.exe" } else { "suite" });
    assert!(
        build.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(executable.is_file());

    fs::remove_file(root.join("suite.fpasworkspace")).expect("workspace must be removable");
    fs::remove_dir_all(root.join("apps")).expect("projects and sources must be removable");
    let output = Command::new(&executable)
        .current_dir(&root)
        .output()
        .expect("workspace application must start");
    fs::remove_dir_all(&root).expect("temporary directory must be removed");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "workspace bundle\n"
    );
}
