//! Relocatable bytecode objects stored in compiled-unit artifacts.

mod relocation;

use std::collections::BTreeMap;
use std::fmt;

use fpas_bytecode::{Chunk, Op, Value};

pub use relocation::{Relocation, RelocationKind, collect_relocations};

/// Constant-pool value supported in a relocatable object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ObjectConstant {
    /// Signed integer.
    Integer(i64),
    /// IEEE-754 bit representation.
    Real(u64),
    /// Boolean.
    Boolean(bool),
    /// UTF-8 string.
    String(String),
    /// Procedure result value.
    Unit,
    /// Named non-capturing function value.
    Function {
        /// Canonical callable name.
        name: String,
        /// Whether calls are restricted to the creating task.
        task_bound: bool,
    },
}

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
pub struct RelocatableObject {
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

impl RelocatableObject {
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
            .map(ObjectConstant::from_value)
            .collect::<Result<Vec<_>, _>>()?;
        let locations = chunk
            .locations()
            .iter()
            .map(|location| ObjectLocation {
                line: location.line,
                column: location.column,
                source_id: location.source_id,
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
    /// Compiler emitted a runtime-only value into the persistent constant pool.
    UnsupportedConstant(String),
}

impl fmt::Display for ObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid relocatable object: {self:?}")
    }
}

impl std::error::Error for ObjectError {}

/// Encode and validate a relocatable object deterministically.
pub fn encode_object(object: &RelocatableObject) -> Result<Vec<u8>, ObjectError> {
    object.validate()?;
    serde_json::to_vec(object).map_err(|error| ObjectError::Encode(error.to_string()))
}

/// Decode and validate a relocatable object.
pub fn decode_object(bytes: &[u8]) -> Result<RelocatableObject, ObjectError> {
    let object: RelocatableObject =
        serde_json::from_slice(bytes).map_err(|error| ObjectError::Decode(error.to_string()))?;
    object.validate()?;
    Ok(object)
}

impl ObjectConstant {
    fn from_value(value: &Value) -> Result<Self, ObjectError> {
        match value {
            Value::Integer(value) => Ok(Self::Integer(*value)),
            Value::Real(value) => Ok(Self::Real(value.to_bits())),
            Value::Boolean(value) => Ok(Self::Boolean(*value)),
            Value::Str(value) => Ok(Self::String(value.to_string())),
            Value::Unit => Ok(Self::Unit),
            Value::Function {
                name,
                captures,
                task_bound,
            } if captures.is_empty() => Ok(Self::Function {
                name: name.clone(),
                task_bound: *task_bound,
            }),
            other => Err(ObjectError::UnsupportedConstant(
                other.type_name().to_string(),
            )),
        }
    }

    /// Convert the persistent constant into its runtime bytecode value.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Integer(value) => Value::Integer(*value),
            Self::Real(bits) => Value::Real(f64::from_bits(*bits)),
            Self::Boolean(value) => Value::Boolean(*value),
            Self::String(value) => Value::Str(value.clone().into()),
            Self::Unit => Value::Unit,
            Self::Function { name, task_bound } => Value::Function {
                name: name.clone(),
                captures: Vec::new(),
                task_bound: *task_bound,
            },
        }
    }
}
