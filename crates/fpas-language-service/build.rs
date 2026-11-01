//! Build the shared analysis identity once for both adapters.

#[path = "build/analysis_identity.rs"]
mod analysis_identity;
#[path = "src/analysis/identity/encoding.rs"]
mod encoding;

fn main() -> std::io::Result<()> {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| std::io::Error::other("Missing analysis crate directory."))?;
    analysis_identity::emit(std::path::Path::new(&manifest))
}
