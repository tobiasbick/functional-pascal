//! Temporary stack-chunk objects retained until the P9 production cutover.

use std::collections::BTreeMap;
use std::fmt;

use fpas_bytecode::{Chunk, Op, PersistentValue};

pub use super::chunk_relocation::{Relocation, RelocationKind, collect_relocations};

/// Persistent constant-pool value used by relocatable unit objects.
pub type ObjectConstant = PersistentValue;

/// Source location independent of a process-local source-path table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectLocation {
    /// One-based line.
    pub line: u32,
    /// One-based column.
    pub column: u32,
    /// Source ID assigned by the owning build graph.
    pub source_id: u32,
}

/// Callable entry in an object-local instruction stream.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectFunction {
    /// Object-local instruction offset.
    pub code_start: u32,
    /// Positional argument count.
    pub arity: u8,
}

/// Category of a named definition or import.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefinitionKind {
    /// Module global or constant.
    Global,
    /// Function or procedure.
    Callable,
}

/// One named definition emitted by an object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectDefinition {
    /// Canonical fully qualified definition name.
    pub name: String,
    /// Runtime definition category.
    pub kind: DefinitionKind,
    /// Whether another object may resolve an import to this definition.
    pub public: bool,
}

/// One named definition required from another object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectImport {
    /// Canonical fully qualified definition name.
    pub name: String,
    /// Expected runtime definition category.
    pub kind: DefinitionKind,
}

/// Independently compiled instruction stream with object-local indices.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChunkObject {
    /// Canonical owner unit, or the program name for the root object.
    pub owner: String,
    /// Object-local instruction stream ending in one `Halt`.
    pub code: Vec<Op>,
    /// Object-local constant pool.
    pub constants: Vec<ObjectConstant>,
    /// Source location parallel to `code`.
    pub locations: Vec<ObjectLocation>,
    /// Callable names mapped to object-local entries.
    pub functions: BTreeMap<String, ObjectFunction>,
    /// Public and private definitions supplied by the object.
    pub definitions: Vec<ObjectDefinition>,
    /// External definitions referenced by the object.
    pub imports: Vec<ObjectImport>,
    /// Complete operand relocation table.
    pub relocations: Vec<Relocation>,
}

impl ChunkObject {
    /// Convert a compiler chunk whose indices are still object-local.
    pub fn from_chunk(
        owner: impl Into<String>,
        chunk: &Chunk,
        definitions: Vec<ObjectDefinition>,
        imports: Vec<ObjectImport>,
    ) -> Result<Self, ObjectError> {
        let code = chunk.code().to_vec();
        let constants = chunk
            .constants()
            .iter()
            .map(|value| {
                ObjectConstant::from_value(value).map_err(|error| match error {
                    fpas_bytecode::PersistentValueError::UnsupportedRuntimeValue(value_type) => {
                        ObjectError::UnsupportedConstant(value_type)
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let locations = chunk
            .locations()
            .iter()
            .map(|location| ObjectLocation {
                line: location.line(),
                column: location.column(),
                source_id: location.source_id(),
            })
            .collect();
        let functions = chunk
            .functions()
            .iter()
            .map(|(name, (offset, arity))| {
                let code_start =
                    u32::try_from(*offset).map_err(|_| ObjectError::CodeSize(chunk.len()))?;
                Ok((
                    name.clone(),
                    ObjectFunction {
                        code_start,
                        arity: *arity,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, ObjectError>>()?;
        let object = Self {
            owner: owner.into(),
            relocations: collect_relocations(&code),
            code,
            constants,
            locations,
            functions,
            definitions,
            imports,
        };
        object.validate()?;
        Ok(object)
    }

    /// Validate local indices, relocation coverage, and structural invariants.
    ///
    /// # Errors
    ///
    /// Returns [`ObjectError`] when the object is not safe to encode or link.
    pub fn validate(&self) -> Result<(), ObjectError> {
        if self.code.len() > u32::MAX as usize {
            return Err(ObjectError::CodeSize(self.code.len()));
        }
        if self.code.len() != self.locations.len() {
            return Err(ObjectError::LocationCount {
                code: self.code.len(),
                locations: self.locations.len(),
            });
        }
        if !matches!(self.code.last(), Some(Op::Halt)) {
            return Err(ObjectError::MissingHalt);
        }
        if let Some(instruction) = self.code[..self.code.len() - 1]
            .iter()
            .position(|op| matches!(op, Op::Halt))
        {
            return Err(ObjectError::InternalHalt { instruction });
        }
        for (name, function) in &self.functions {
            if function.code_start as usize >= self.code.len() {
                return Err(ObjectError::FunctionOffset {
                    name: name.clone(),
                    offset: function.code_start,
                    code: self.code.len(),
                });
            }
        }
        let expected = collect_relocations(&self.code);
        if expected != self.relocations {
            return Err(ObjectError::RelocationCoverage);
        }
        for relocation in &self.relocations {
            match relocation.kind {
                RelocationKind::Constant { index, .. }
                    if index as usize >= self.constants.len() =>
                {
                    return Err(ObjectError::ConstantIndex {
                        instruction: relocation.instruction,
                        index,
                        constants: self.constants.len(),
                    });
                }
                RelocationKind::CodeAddress { target } if target as usize > self.code.len() => {
                    return Err(ObjectError::CodeTarget {
                        instruction: relocation.instruction,
                        target,
                        code: self.code.len(),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Invalid relocatable object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectError {
    /// Instruction stream cannot be addressed with bytecode offsets.
    CodeSize(usize),
    /// Instruction and location streams differ in length.
    LocationCount {
        /// Instruction count.
        code: usize,
        /// Location count.
        locations: usize,
    },
    /// Object does not end in `Halt`.
    MissingHalt,
    /// Object contains a `Halt` before the final instruction.
    InternalHalt {
        /// Zero-based instruction offset.
        instruction: usize,
    },
    /// Callable entry points outside the local stream.
    FunctionOffset {
        /// Callable name.
        name: String,
        /// Invalid local offset.
        offset: u32,
        /// Instruction count.
        code: usize,
    },
    /// Recorded relocations do not exactly match operands in the instruction stream.
    RelocationCoverage,
    /// Constant operand is outside the local pool.
    ConstantIndex {
        /// Instruction containing the operand.
        instruction: u32,
        /// Invalid constant index.
        index: u16,
        /// Constant count.
        constants: usize,
    },
    /// Jump target is outside the local stream.
    CodeTarget {
        /// Instruction containing the target.
        instruction: u32,
        /// Invalid target.
        target: u32,
        /// Instruction count.
        code: usize,
    },
    /// Object serialization failed.
    Encode(String),
    /// Object decoding failed.
    Decode(String),
    /// Encoded object exceeds the compiled-unit payload limit.
    PayloadSize {
        /// Encoded or requested size.
        size: usize,
        /// Largest accepted size.
        maximum: usize,
    },
    /// Compiler emitted a runtime-only value into the persistent constant pool.
    UnsupportedConstant(String),
}

impl fmt::Display for ObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InternalHalt { instruction } => write!(
                formatter,
                "invalid relocatable object: internal Halt at instruction {instruction}; only the final instruction may halt"
            ),
            _ => write!(formatter, "invalid relocatable object: {self:?}"),
        }
    }
}

impl std::error::Error for ObjectError {}

/// Encode and validate a relocatable object deterministically.
///
/// # Errors
///
/// Returns [`ObjectError`] when validation or serialization fails, or when
/// the encoded object exceeds the compiled-unit payload limit.
pub fn encode_chunk_object(object: &ChunkObject) -> Result<Vec<u8>, ObjectError> {
    object.validate()?;
    let bytes =
        serde_json::to_vec(object).map_err(|error| ObjectError::Encode(error.to_string()))?;
    check_payload_size(bytes.len())?;
    Ok(bytes)
}

/// Decode and validate a relocatable object.
///
/// # Errors
///
/// Returns [`ObjectError`] when the payload is oversized, malformed, or
/// violates relocatable-object invariants.
pub fn decode_chunk_object(bytes: &[u8]) -> Result<ChunkObject, ObjectError> {
    check_payload_size(bytes.len())?;
    let object: ChunkObject =
        serde_json::from_slice(bytes).map_err(|error| ObjectError::Decode(error.to_string()))?;
    object.validate()?;
    Ok(object)
}

fn check_payload_size(size: usize) -> Result<(), ObjectError> {
    crate::format::check_payload_size("object", size).map_err(|_| ObjectError::PayloadSize {
        size,
        maximum: crate::format::MAX_PAYLOAD_BYTES,
    })
}

#[cfg(test)]
mod tests {
    use super::check_payload_size;
    use crate::format::MAX_PAYLOAD_BYTES;

    #[test]
    fn direct_object_codec_enforces_payload_limit() {
        assert!(check_payload_size(MAX_PAYLOAD_BYTES).is_ok());
        assert!(check_payload_size(MAX_PAYLOAD_BYTES + 1).is_err());
    }
}
