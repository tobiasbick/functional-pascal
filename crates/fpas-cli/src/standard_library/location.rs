//! Locate the library belonging to a built or installed CLI.

use std::path::{Path, PathBuf};

pub(super) fn resolve(
    executable: &Path,
    build_profile: &Path,
    build_library: &Path,
) -> Option<PathBuf> {
    let executable_dir = executable.parent()?;
    // Cargo may restore a cached executable without rerunning its build script.
    // Shared final output directories can therefore contain another checkout's library.
    // Dep-info beside the executable also identifies command-line --target-dir outputs.
    if (executable_dir == build_profile
        || executable.with_extension("d").is_file()
        || build_library
            .parent()
            .is_some_and(|profile| executable_dir == profile.join("deps")))
        && build_library.join("stdlib.fpasprj").is_file()
    {
        return Some(build_library.to_path_buf());
    }
    let adjacent = executable_dir.join("lib");
    if adjacent.join("stdlib.fpasprj").is_file() {
        return Some(adjacent);
    }
    let development_library = executable_dir.parent()?.join("lib");
    development_library
        .join("stdlib.fpasprj")
        .is_file()
        .then_some(development_library)
}
