//! Expected compiled-program identity derived from authoritative build inputs.

use fpas_program::{Digest, ProgramIdentity};

use crate::{BuildOptions, BuiltUnits};

pub(super) fn expected(
    source: &[u8],
    units: &BuiltUnits,
    options: &BuildOptions,
) -> ProgramIdentity {
    ProgramIdentity {
        compiler_version: options.compiler_version.clone(),
        bytecode_version: options.bytecode_version,
        source_hash: Digest::of(source),
        options_hash: Digest::from_bytes(*options.options_hash.as_bytes()),
        units: units.linked_units.clone(),
    }
}
