//! In-memory parse cache used while loading a project dependency tree.
//!
//! Avoids lexing and parsing the same `.fpas` file multiple times during manifest
//! validation (`[exports]`, unit checks, main validation, merged source validation).

use crate::paths::canonical_source_path;
use crate::source::parse_compilation_unit_file;
use fpas_parser::CompilationUnit;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Parsed compilation units keyed by canonical source path.
#[derive(Debug, Default)]
pub(crate) struct ParsedSourceCache {
    entries: HashMap<PathBuf, (CompilationUnit, Vec<String>)>,
    parse_misses: usize,
}

impl ParsedSourceCache {
    /// Creates an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a parsed compilation unit, reusing a prior result for the same file.
    pub fn parse(
        &mut self,
        path: &Path,
        source_id: u32,
    ) -> Result<(CompilationUnit, Vec<String>), String> {
        let key = canonical_source_path(path);
        if let Some(entry) = self.entries.get(&key) {
            return Ok(entry.clone());
        }

        self.parse_misses += 1;
        let parsed = parse_compilation_unit_file(path, source_id)?;
        self.entries.insert(key, parsed.clone());
        Ok(parsed)
    }

    /// Number of distinct source files parsed so far (cache misses).
    #[cfg(test)]
    pub fn parse_miss_count(&self) -> usize {
        self.parse_misses
    }
}

#[cfg(test)]
mod tests {
    use super::ParsedSourceCache;
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_file(name: &str, contents: &str) -> Result<std::path::PathBuf, std::io::Error> {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "fpas-parse-cache-{name}-{}-{id}.fpas",
            std::process::id()
        ));
        fs::write(&path, contents)?;
        Ok(path)
    }

    #[test]
    fn parse_is_cached_by_canonical_path() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_file("unit", "unit Demo.Core;\n")?;
        let mut cache = ParsedSourceCache::new();

        cache.parse(&path, 0)?;
        cache.parse(&path, 0)?;

        assert_eq!(cache.parse_miss_count(), 1);
        fs::remove_file(path).ok();
        Ok(())
    }

    #[test]
    fn parse_cache_hits_across_equivalent_paths() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!(
            "fpas-parse-cache-dir-{}-{}",
            std::process::id(),
            AtomicU64::new(1).fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir)?;
        let file_name = "core.fpas";
        let relative = dir.join(file_name);
        fs::write(&relative, "unit Demo.Core;\n")?;

        let absolute = fs::canonicalize(&relative)?;
        let mut cache = ParsedSourceCache::new();
        cache.parse(Path::new(&relative), 0)?;
        cache.parse(&absolute, 0)?;

        assert_eq!(cache.parse_miss_count(), 1);
        fs::remove_dir_all(dir).ok();
        Ok(())
    }
}
