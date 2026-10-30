//! Build-time location of the bundled FPAS source library.

/// Source tree belonging to this Cargo dependency and checkout.
pub const SOURCE_ROOT: &str = env!("CARGO_MANIFEST_DIR");
