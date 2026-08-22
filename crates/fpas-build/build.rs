//! Emits the compiler identity consumed by incremental build artifacts.

#[path = "build/compiler_identity.rs"]
mod compiler_identity;

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compiler_identity::emit(Path::new(env!("CARGO_MANIFEST_DIR")))?;
    Ok(())
}
