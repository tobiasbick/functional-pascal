//! Untrusted and verified register executable aggregates.

mod provenance;

pub use provenance::{ExecutableIdentity, FunctionTarget};

use std::sync::Arc;

use crate::{
    Constant, DebugType, DecodedInstruction, DistinctTypeName, EnumLayout, EnumVariant, FunctionId,
    FunctionInfo, GlobalInfo, Instruction, InstructionAddress, RecordLayout, SourceMap,
    StringTable, ValidationError, ValidationErrorKind,
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
    /// Source-visible distinct type names used only by debugger conversions.
    pub distinct_types: Vec<DistinctTypeName>,
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
        let function_names = self
            .functions
            .iter()
            .map(|function| Arc::from(self.strings.get(function.name).unwrap_or_default()))
            .collect();
        Ok(VerifiedExecutable {
            identity: ExecutableIdentity::new(),
            executable: self,
            decoded,
            function_names,
        })
    }
}

/// Register executable whose structural and operand invariants have been checked.
#[derive(Debug)]
pub struct VerifiedExecutable {
    identity: ExecutableIdentity,
    executable: Executable,
    decoded: Vec<DecodedInstruction>,
    // Shared diagnostic names for first-class function values, indexed by `FunctionId`.
    function_names: Vec<Arc<str>>,
}

impl VerifiedExecutable {
    /// Borrow the runtime identity of this independently verified code instance.
    ///
    /// Sharing an `Arc<VerifiedExecutable>` preserves this identity. Cloning the
    /// verified image itself creates an independent instance with a fresh identity.
    #[must_use]
    #[inline]
    pub fn identity(&self) -> &ExecutableIdentity {
        &self.identity
    }

    /// Bind a function index to its validated executable and canonical name.
    ///
    /// Returns `None` for an index outside the verified function table. Runtime
    /// function values require this target instead of a caller-supplied code token.
    #[must_use]
    pub fn function_target(&self, function: FunctionId) -> Option<FunctionTarget> {
        self.function_name(function)
            .map(|name| FunctionTarget::new(self.identity.clone(), function, Arc::clone(name)))
    }

    /// Borrow the immutable validated executable.
    #[must_use]
    #[inline]
    pub const fn executable(&self) -> &Executable {
        &self.executable
    }

    /// Borrow instructions prepared for the interpreter after validation.
    #[must_use]
    #[inline]
    pub fn decoded(&self) -> &[DecodedInstruction] {
        &self.decoded
    }

    /// Borrow the shared name that first-class values of `function` carry.
    #[must_use]
    #[inline]
    pub fn function_name(&self, function: FunctionId) -> Option<&Arc<str>> {
        self.function_names.get(usize::from(function.get()))
    }

    /// Consume the proof wrapper and return the untrusted candidate representation.
    #[must_use]
    pub fn into_unverified(self) -> Executable {
        self.executable
    }
}

impl Clone for VerifiedExecutable {
    fn clone(&self) -> Self {
        Self {
            identity: ExecutableIdentity::new(),
            executable: self.executable.clone(),
            decoded: self.decoded.clone(),
            function_names: self.function_names.clone(),
        }
    }
}

// Artifact and metadata comparisons remain structural. Runtime admission uses
// the separate opaque identity and never derives permission from this equality.
impl PartialEq for VerifiedExecutable {
    fn eq(&self, other: &Self) -> bool {
        self.executable == other.executable
            && self.decoded == other.decoded
            && self.function_names == other.function_names
    }
}

impl Eq for VerifiedExecutable {}
