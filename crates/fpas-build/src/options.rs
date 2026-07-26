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
            compiler_version: concat!(
                env!("CARGO_PKG_VERSION"),
                "-",
                env!("FPAS_COMPILER_BUILD_ID")
            )
            .to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            options_hash: Digest::of(b"fpas-default-compilation-options-v1"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BuildOptions;

    #[test]
    fn default_identity_contains_the_package_version_and_source_fingerprint() {
        let identity = BuildOptions::default().compiler_version;
        let prefix = concat!(env!("CARGO_PKG_VERSION"), "-source-");
        let fingerprint = identity
            .strip_prefix(prefix)
            .expect("compiler identity must start with its package version");

        assert_eq!(fingerprint.len(), 64);
        assert!(fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
