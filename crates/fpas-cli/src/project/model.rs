use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Program,
    Library,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProject {
    pub kind: ProjectKind,
    pub main: Option<PathBuf>,
    pub source_files: Vec<PathBuf>,
    pub warnings: Vec<String>,
}
