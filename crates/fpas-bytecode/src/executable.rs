//! Untrusted and verified register executable aggregates.

use crate::{
    Constant, EnumLayout, EnumVariant, FunctionId, FunctionInfo, GlobalInfo, Instruction,
    RecordLayout, SourceMap, StringTable, ValidationError,
};

/// Complete untrusted register-bytecode candidate produced by a compiler or decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    /// Contiguous packed instruction stream.
    pub code: Vec<Instruction>,
    /// Dense function table; vector indices are [`FunctionId`] values.
    pub functions: Vec<FunctionInfo>,
    /// Persistent runtime-independent constants.
    pub constants: Vec<Constant>,
    /// Deterministic UTF-8 strings used by metadata.
    pub strings: StringTable,
    /// Dense global slot declarations.
    pub globals: Vec<GlobalInfo>,
    /// Dense record layout table.
    pub records: Vec<RecordLayout>,
    /// Dense enum type table.
    pub enums: Vec<EnumLayout>,
    /// Executable-wide enum variant table.
    pub enum_variants: Vec<EnumVariant>,
    /// Sparse diagnostic source locations.
    pub source_map: SourceMap,
    /// Root initializer and entry function, required to be function zero.
    pub entry: FunctionId,
}

impl Executable {
    /// Validate every executable invariant and prevent unchecked VM admission.
    ///
    /// # Errors
    ///
    /// Returns a contextual [`ValidationError`] for the first deterministic violation.
    pub fn verify(self) -> Result<VerifiedExecutable, ValidationError> {
        crate::validate::validate(&self)?;
        Ok(VerifiedExecutable { executable: self })
    }
}

/// Register executable whose structural and operand invariants have been checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedExecutable {
    executable: Executable,
}

impl VerifiedExecutable {
    /// Borrow the immutable validated executable.
    #[must_use]
    pub const fn executable(&self) -> &Executable {
        &self.executable
    }

    /// Consume the proof wrapper and return the untrusted candidate representation.
    #[must_use]
    pub fn into_unverified(self) -> Executable {
        self.executable
    }
}
