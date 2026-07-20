use std::io;
use std::path::Path;

mod stdlib_sync;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_dir.join("../../lib");
    println!("cargo:rerun-if-changed={}", source_root.display());

    let out_dir = std::env::var_os("OUT_DIR")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo must set OUT_DIR"))?;
    let profile_dir = Path::new(&out_dir).ancestors().nth(3).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cargo OUT_DIR must be below the target profile directory",
        )
    })?;
    stdlib_sync::replace_tree(&source_root, &profile_dir.join("lib"))?;
    Ok(())
}
