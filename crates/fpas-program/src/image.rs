//! In-memory model and payload conversion for executable program images.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::Path;

use fpas_bytecode::{
    Chunk, ExecutableError, Op, PersistentValue, PersistentValueError, SourceLocation,
    validate_executable,
};
use serde::{Deserialize, Serialize};

use crate::ProgramIdentity;

/// Invalid in-memory program image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    /// Required identity field is empty.
    EmptyIdentityField(&'static str),
    /// Image bytecode is incompatible with this runtime.
    BytecodeVersion {
        /// Version recorded in the image.
        image: u32,
        /// Version understood by this runtime.
        runtime: u32,
    },
    /// A reachable unit appears more than once.
    DuplicateUnit(String),
    /// A source path is empty.
    EmptySourcePath(usize),
    /// A source path contains machine-specific absolute metadata.
    AbsoluteSourcePath(String),
    /// An instruction location refers outside the source-path table.
    SourceId {
        /// Instruction offset containing the location.
        instruction: usize,
        /// Referenced source identifier.
        source_id: u32,
        /// Number of source paths in the image.
        source_paths: usize,
    },
    /// Executable bytecode structure is invalid.
    Executable(ExecutableError),
    /// A constant contains runtime-only state.
    PersistentValue(PersistentValueError),
    /// A decoded source location is not one-based.
    InvalidLocation {
        /// Instruction offset containing the location.
        instruction: usize,
        /// Encoded line.
        line: u32,
        /// Encoded column.
        column: u32,
    },
    /// A decoded constant pool contains a duplicate that would change its indices.
    DuplicateConstant {
        /// Encoded constant-pool index.
        index: usize,
        /// Existing index returned by the chunk.
        existing: u16,
    },
    /// A decoded constant pool exceeds bytecode limits.
    ConstantPool(String),
    /// The JSON payload could not be encoded.
    PayloadEncode(String),
    /// The JSON payload could not be decoded.
    PayloadDecode(String),
}

impl fmt::Display for ImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid compiled program image: {self:?}")
    }
}

impl std::error::Error for ImageError {}

/// Complete executable bytecode plus its build identity and diagnostic sources.
pub struct ProgramImage {
    identity: ProgramIdentity,
    source_paths: Vec<String>,
    chunk: Chunk,
}

impl ProgramImage {
    /// Construct and validate a complete program image.
    pub fn new(
        identity: ProgramIdentity,
        source_paths: Vec<String>,
        chunk: Chunk,
    ) -> Result<Self, ImageError> {
        let image = Self {
            identity,
            source_paths,
            chunk,
        };
        image.validate()?;
        Ok(image)
    }

    /// Return the recorded build identity.
    #[must_use]
    pub fn identity(&self) -> &ProgramIdentity {
        &self.identity
    }

    /// Return the relative source-path table used by diagnostics.
    #[must_use]
    pub fn source_paths(&self) -> &[String] {
        &self.source_paths
    }

    /// Return the executable bytecode.
    #[must_use]
    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    /// Consume the image and return its executable bytecode.
    #[must_use]
    pub fn into_chunk(self) -> Chunk {
        self.chunk
    }

    pub(crate) fn validate(&self) -> Result<(), ImageError> {
        validate_identity(&self.identity)?;
        validate_source_paths(&self.source_paths)?;
        validate_executable(&self.chunk).map_err(ImageError::Executable)?;
        validate_locations(&self.chunk, self.source_paths.len())?;
        for value in self.chunk.constants() {
            PersistentValue::from_value(value).map_err(ImageError::PersistentValue)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct EncodedChunk {
    code: Vec<Op>,
    constants: Vec<PersistentValue>,
    locations: Vec<EncodedLocation>,
    functions: BTreeMap<String, EncodedFunction>,
}

#[derive(Serialize, Deserialize)]
struct EncodedLocation {
    line: u32,
    column: u32,
    source_id: u32,
}

#[derive(Serialize, Deserialize)]
struct EncodedFunction {
    code_start: u32,
    arity: u8,
}

pub(crate) fn encode_payload(image: &ProgramImage) -> Result<Vec<u8>, ImageError> {
    image.validate()?;
    let constants = image
        .chunk
        .constants()
        .iter()
        .map(PersistentValue::from_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ImageError::PersistentValue)?;
    let locations = image
        .chunk
        .locations()
        .iter()
        .map(|location| EncodedLocation {
            line: location.line,
            column: location.column,
            source_id: location.source_id,
        })
        .collect();
    let functions = image
        .chunk
        .functions()
        .iter()
        .map(|(name, (code_start, arity))| {
            let code_start = u32::try_from(*code_start).map_err(|_| {
                ImageError::Executable(ExecutableError::FunctionOffset {
                    name: name.clone(),
                    offset: *code_start,
                    code: image.chunk.len(),
                })
            })?;
            Ok((
                name.clone(),
                EncodedFunction {
                    code_start,
                    arity: *arity,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ImageError>>()?;
    let payload = EncodedChunk {
        code: image.chunk.code().to_vec(),
        constants,
        locations,
        functions,
    };
    serde_json::to_vec(&payload).map_err(|error| ImageError::PayloadEncode(error.to_string()))
}

pub(crate) fn decode_payload(
    identity: ProgramIdentity,
    source_paths: Vec<String>,
    bytes: &[u8],
) -> Result<ProgramImage, ImageError> {
    let payload: EncodedChunk = serde_json::from_slice(bytes)
        .map_err(|error| ImageError::PayloadDecode(error.to_string()))?;
    let chunk = decode_chunk(payload)?;
    ProgramImage::new(identity, source_paths, chunk)
}

fn decode_chunk(payload: EncodedChunk) -> Result<Chunk, ImageError> {
    if payload.code.len() != payload.locations.len() {
        return Err(ImageError::Executable(ExecutableError::Chunk(
            fpas_bytecode::ChunkError::CodeLocationLengthMismatch {
                code_len: payload.code.len(),
                locations_len: payload.locations.len(),
            },
        )));
    }

    let mut chunk = Chunk::new();
    for (index, constant) in payload.constants.iter().enumerate() {
        let actual = chunk
            .add_constant(constant.to_value())
            .map_err(|error| ImageError::ConstantPool(error.to_string()))?;
        if actual as usize != index {
            return Err(ImageError::DuplicateConstant {
                index,
                existing: actual,
            });
        }
    }
    for (instruction, (op, location)) in payload.code.into_iter().zip(payload.locations).enumerate()
    {
        if location.line == 0 || location.column == 0 {
            return Err(ImageError::InvalidLocation {
                instruction,
                line: location.line,
                column: location.column,
            });
        }
        chunk.emit(
            op,
            SourceLocation::new_with_source(location.line, location.column, location.source_id),
        );
    }
    for (name, function) in payload.functions {
        chunk.insert_function(name, function.code_start as usize, function.arity);
    }
    Ok(chunk)
}

fn validate_identity(identity: &ProgramIdentity) -> Result<(), ImageError> {
    if identity.compiler_version.trim().is_empty() {
        return Err(ImageError::EmptyIdentityField("compiler_version"));
    }
    if identity.bytecode_version != fpas_bytecode::BYTECODE_VERSION {
        return Err(ImageError::BytecodeVersion {
            image: identity.bytecode_version,
            runtime: fpas_bytecode::BYTECODE_VERSION,
        });
    }
    let mut units = HashSet::with_capacity(identity.units.len());
    for unit in &identity.units {
        if unit.unit_name.trim().is_empty() {
            return Err(ImageError::EmptyIdentityField("unit_name"));
        }
        let canonical = unit.unit_name.to_ascii_lowercase();
        if !units.insert(canonical) {
            return Err(ImageError::DuplicateUnit(unit.unit_name.clone()));
        }
    }
    Ok(())
}

fn validate_source_paths(source_paths: &[String]) -> Result<(), ImageError> {
    for (index, source_path) in source_paths.iter().enumerate() {
        if source_path.trim().is_empty() {
            return Err(ImageError::EmptySourcePath(index));
        }
        if Path::new(source_path).is_absolute() {
            return Err(ImageError::AbsoluteSourcePath(source_path.clone()));
        }
    }
    Ok(())
}

fn validate_locations(chunk: &Chunk, source_path_count: usize) -> Result<(), ImageError> {
    for (instruction, location) in chunk.locations().iter().enumerate() {
        if location.source_id as usize >= source_path_count {
            return Err(ImageError::SourceId {
                instruction,
                source_id: location.source_id,
                source_paths: source_path_count,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Digest;

    fn identity() -> ProgramIdentity {
        ProgramIdentity {
            compiler_version: "test-compiler".to_string(),
            bytecode_version: fpas_bytecode::BYTECODE_VERSION,
            source_hash: Digest::of(b"source"),
            options_hash: Digest::of(b"options"),
            units: Vec::new(),
        }
    }

    #[test]
    fn payload_decoder_rejects_unknown_opcode_tag() {
        let payload = br#"{"code":["Unknown"],"constants":[],"locations":[],"functions":{}}"#;

        assert!(matches!(
            decode_payload(identity(), vec!["main.fpas".to_string()], payload),
            Err(ImageError::PayloadDecode(_))
        ));
    }

    #[test]
    fn payload_decoder_rejects_zero_based_location() {
        let payload = br#"{"code":["Halt"],"constants":[],"locations":[{"line":0,"column":1,"source_id":0}],"functions":{}}"#;

        assert_eq!(
            decode_payload(identity(), vec!["main.fpas".to_string()], payload).err(),
            Some(ImageError::InvalidLocation {
                instruction: 0,
                line: 0,
                column: 1,
            })
        );
    }

    #[test]
    fn payload_decoder_rejects_duplicate_constants() {
        let payload = br#"{"code":["Halt"],"constants":[{"Integer":1},{"Integer":1}],"locations":[{"line":1,"column":1,"source_id":0}],"functions":{}}"#;

        assert_eq!(
            decode_payload(identity(), vec!["main.fpas".to_string()], payload).err(),
            Some(ImageError::DuplicateConstant {
                index: 1,
                existing: 0,
            })
        );
    }
}
