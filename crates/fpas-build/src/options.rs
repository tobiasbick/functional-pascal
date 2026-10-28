//! Stable compilation identity used for sidecar validation.

use fpas_unit::Digest;

/// Compatibility and option identity for one build invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    /// Generate contract conditions and checks throughout the complete dependency graph.
    pub contracts: bool,
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
            contracts: true,
            compiler_version: concat!(
                env!("CARGO_PKG_VERSION"),
                "-",
                env!("FPAS_COMPILER_BUILD_ID")
            )
            .to_string(),
            bytecode_version: fpas_code::BYTECODE_VERSION,
            options_hash: Digest::of(b"fpas-default-compilation-options-v1"),
        }
    }
}

impl BuildOptions {
    /// Bind every code-generation policy to the reusable artifact identity.
    #[must_use]
    pub fn effective_options_hash(&self) -> Digest {
        if self.contracts {
            return self.options_hash;
        }
        let mut bytes = [0; Digest::LENGTH + 1];
        bytes[..Digest::LENGTH].copy_from_slice(self.options_hash.as_bytes());
        bytes[Digest::LENGTH] = u8::from(self.contracts);
        Digest::of(bytes)
    }

    /// Compiler policy used by every root and dependency in this invocation.
    #[must_use]
    pub const fn compile_options(&self) -> fpas_compiler::CompileOptions {
        fpas_compiler::CompileOptions {
            contracts: self.contracts,
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
        let fingerprint = identity.strip_prefix(prefix);

        assert!(
            fingerprint.is_some(),
            "compiler identity must start with its package version"
        );
        let fingerprint = fingerprint.unwrap_or_default();
        assert_eq!(fingerprint.len(), 64);
        assert!(fingerprint.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
