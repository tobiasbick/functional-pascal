use std::path::PathBuf;

/// Kind of `.fpasprj` project described in `docs/pascal/10-projects.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// Executable project with a single `program` entry file.
    Program,
    /// Library project that contains only `unit` source files.
    Library,
}

/// Resolved project metadata and source file set ready for linking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProject {
    /// Declared project kind.
    pub kind: ProjectKind,
    /// Main program file for executable projects.
    pub main: Option<PathBuf>,
    /// Validated user-unit source files included by the project.
    pub source_files: Vec<PathBuf>,
    /// Non-fatal loading warnings such as duplicate include entries.
    pub warnings: Vec<String>,
}
