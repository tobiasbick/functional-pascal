//! Disk-backed user units and an editor overlay with the real source standard library.

use std::fmt::Write;
use std::path::PathBuf;

use super::super::fixture_directory::FixtureDirectory;
use fpas_language_service::LanguageService;

/// A project with an open main buffer and unopened disk-backed sibling units.
pub(super) struct Fixture {
    pub(super) service: LanguageService,
    pub(super) main: PathBuf,
    pub(super) source: String,
    _directory: FixtureDirectory,
}

impl Fixture {
    /// Writes benchmark sources under the workspace scratch root and loads editor context.
    pub(super) fn create(units: usize) -> Result<Self, String> {
        let root = std::env::current_dir().map_err(|error| error.to_string())?;
        let scratch = FixtureDirectory::create(&root)?;
        let directory = scratch.path();
        let manifest = directory.join("queries.fpasprj");
        std::fs::write(&manifest, "[project]\nname = \"queries\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n").map_err(|error| error.to_string())?;
        let mut source = String::from("program ProjectQueries;\nuses Std.Str");
        for unit in 0..units {
            write!(source, ", Bench.U{unit}").map_err(|error| error.to_string())?;
            std::fs::write(directory.join(format!("unit{unit}.fpas")), format!("unit Bench.U{unit};\npublic function Answer{unit}(): integer;\nbegin return {unit} end;\n")).map_err(|error| error.to_string())?;
        }
        source.push_str(";\nbegin\nvar Count: integer := Std.Str.Length('é😀');\n");
        for unit in 0..units {
            writeln!(source, "var Value{unit}: integer := Answer{unit}();")
                .map_err(|error| error.to_string())?;
        }
        source.push_str("end.\n");
        let main = directory.join("main.fpas");
        std::fs::write(&main, &source).map_err(|error| error.to_string())?;
        let mut service = LanguageService::load_with_standard_library(&manifest, &root.join("lib"))
            .map_err(|error| error.to_string())?;
        if !service.workspace().issues().is_empty() {
            return Err(format!(
                "Benchmark project did not load: {:?}",
                service.workspace().issues()
            ));
        }
        service
            .documents_mut()
            .open_document(&main, 1, source.clone())
            .map_err(|error| error.to_string())?;
        Ok(Self {
            service,
            main,
            source,
            _directory: scratch,
        })
    }
}
