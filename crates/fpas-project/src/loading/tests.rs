use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::own::{load_own_project, validate_project_source_units};
use super::parse_cache::ParsedSourceCache;

fn temp_dir(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-project-cache-{name}-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, text: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}

#[test]
fn export_and_unit_validation_parse_each_library_source_once()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = temp_dir("library");
    let manifest = dir.join("library.fpasprj");
    write(
        &manifest,
        "[project]\nname = \"library\"\nkind = \"library\"\n\n[exports]\nunits = [\"Demo.Api\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    )?;
    write(&dir.join("src/api.fpas"), "unit Demo.Api;\n")?;
    write(&dir.join("src/internal.fpas"), "unit Demo.Internal;\n")?;
    let mut cache = ParsedSourceCache::new();

    let own = load_own_project(&manifest, &mut cache)?;
    assert_eq!(cache.parse_miss_count(), 2);
    validate_project_source_units(own.source_files, &mut Vec::new(), &mut cache)?;

    assert_eq!(cache.parse_miss_count(), 2);
    fs::remove_dir_all(dir).ok();
    Ok(())
}

#[test]
fn main_and_unit_validation_share_one_parse_cache() -> Result<(), Box<dyn std::error::Error>> {
    let dir = temp_dir("program");
    let manifest = dir.join("program.fpasprj");
    write(
        &manifest,
        "[project]\nname = \"program\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    )?;
    write(&dir.join("src/main.fpas"), "program Demo;\nbegin\nend.\n")?;
    write(&dir.join("src/core.fpas"), "unit Demo.Core;\n")?;
    let mut cache = ParsedSourceCache::new();

    let own = load_own_project(&manifest, &mut cache)?;
    assert_eq!(cache.parse_miss_count(), 1);
    validate_project_source_units(own.source_files, &mut Vec::new(), &mut cache)?;

    assert_eq!(cache.parse_miss_count(), 2);
    fs::remove_dir_all(dir).ok();
    Ok(())
}
