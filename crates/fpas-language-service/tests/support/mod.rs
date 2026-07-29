#![allow(
    dead_code,
    reason = "each integration-test binary uses a different subset of fixture helpers"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct TempDirectory {
    path: PathBuf,
}

impl TempDirectory {
    pub fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "fpas-language-service-{label}-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temporary fixture directory must be created");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.path.join(path)
    }

    pub fn write(&self, path: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture parent must be created");
        }
        std::fs::write(&path, source).expect("fixture source must be written");
        path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).ok();
    }
}

pub fn write_program_project(temp: &TempDirectory) -> (PathBuf, PathBuf, PathBuf) {
    let manifest = temp.write(
        "demo.fpasprj",
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let main = temp.write(
        "src/main.fpas",
        "program App;\n\nuses Demo.Math;\n\nbegin\n  var Value: integer := Answer()\nend.\n",
    );
    let unit = temp.write(
        "src/math.fpas",
        "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    );
    (manifest, main, unit)
}
