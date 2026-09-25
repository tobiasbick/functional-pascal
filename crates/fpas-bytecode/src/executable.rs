//! Untrusted and verified register executable aggregates.

use crate::{
    Constant, DebugType, DecodedInstruction, EnumLayout, EnumVariant, FunctionId, FunctionInfo,
    GlobalInfo, Instruction, InstructionAddress, RecordLayout, SharedStr, SourceMap, StringTable,
    ValidationError, ValidationErrorKind,
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
    /// Portable type graph used only by debugger tooling.
    pub debug_types: Vec<DebugType>,
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
        let decoded = self
            .code
            .iter()
            .enumerate()
            .map(|(index, instruction)| {
                DecodedInstruction::decode(*instruction).map_err(|error| ValidationError {
                    function: None,
                    function_name: None,
                    instruction: InstructionAddress::try_from_index(index).ok(),
                    opcode: None,
                    kind: ValidationErrorKind::Instruction(error),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let string_constants = self
            .constants
            .iter()
            .map(|constant| match constant {
                Constant::String(string) => self.strings.get(*string).map(SharedStr::from),
                _ => None,
            })
            .collect();
        Ok(VerifiedExecutable {
            executable: self,
            decoded,
            string_constants,
        })
    }
}

/// Register executable whose structural and operand invariants have been checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedExecutable {
    executable: Executable,
    decoded: Vec<DecodedInstruction>,
    // Shared runtime strings for string constants, indexed like `Executable::constants`.
    string_constants: Vec<Option<SharedStr>>,
}

impl VerifiedExecutable {
    /// Borrow the immutable validated executable.
    #[must_use]
    pub const fn executable(&self) -> &Executable {
        &self.executable
    }

    /// Borrow instructions prepared for the interpreter after validation.
    #[must_use]
    pub fn decoded(&self) -> &[DecodedInstruction] {
        &self.decoded
    }

    /// Borrow the shared runtime string prepared for a string constant.
    ///
    /// Returns `None` when `index` is outside the constant table or names a non-string constant.
    #[must_use]
    pub fn string_constant(&self, index: usize) -> Option<&SharedStr> {
        self.string_constants.get(index).and_then(Option::as_ref)
    }

    /// Consume the proof wrapper and return the untrusted candidate representation.
    #[must_use]
    pub fn into_unverified(self) -> Executable {
        self.executable
    }
}
