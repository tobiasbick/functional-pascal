//! Stable compilation identity used for sidecar validation.

use fpas_unit::Digest;

/// Compatibility and option identity for one build invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    /// Compiler build identity.
    pub compiler_version: String,
    /// Persistent bytecode instruction-set version.
    pub bytecode_version: u32,
    /// Hash of semantic and code-generation options.
    pub options_hash: Digest,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            compiler_version: concat!(env!("CARGO_PKG_VERSION"), "-fpascu-v9").to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            options_hash: Digest::of(b"fpas-default-compilation-options-v1"),
        }
    }
}
