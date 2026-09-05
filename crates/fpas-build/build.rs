//! Emits the compiler identity consumed by incremental build artifacts.

#[path = "build/compiler_identity.rs"]
mod compiler_identity;

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or("Cargo must set CARGO_MANIFEST_DIR")?,
    );
    compiler_identity::emit(&manifest_dir)?;
    Ok(())
}
