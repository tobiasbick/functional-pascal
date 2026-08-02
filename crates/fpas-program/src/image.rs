//! In-memory model and payload conversion for executable program images.

mod payload;
pub(crate) mod resources;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fmt;

use fpas_bytecode::{
    Chunk, ExecutableError, PersistentValue, PersistentValueError, validate_executable,
};

use crate::ProgramIdentity;

pub(crate) use payload::{decode_payload, encode_payload};
use resources::{
    MAX_CONSTANTS, MAX_FUNCTIONS, MAX_INSTRUCTIONS, MAX_LOCATIONS, MAX_TOTAL_STRING_BYTES,
    add_string_bytes, check_resource_size, persistent_string_bytes,
};

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
    /// An in-memory payload resource exceeds its safety limit.
    ResourceLimit {
        /// Logical resource name.
        field: &'static str,
        /// Observed resource size.
        size: usize,
        /// Largest accepted resource size.
        maximum: usize,
    },
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
    /// Construct, canonicalize, and validate a complete program image.
    ///
    /// # Errors
    ///
    /// Returns [`ImageError`] when the identity, source table, executable, or
    /// persistent resources are invalid.
    pub fn new(
        mut identity: ProgramIdentity,
        source_paths: Vec<String>,
        chunk: Chunk,
    ) -> Result<Self, ImageError> {
        for unit in &mut identity.units {
            unit.unit_name.make_ascii_lowercase();
        }
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
        validate_resources(self)?;
        validate_executable(&self.chunk).map_err(ImageError::Executable)?;
        validate_locations(&self.chunk, self.source_paths.len())?;
        Ok(())
    }
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
        if is_portably_absolute(source_path) {
            return Err(ImageError::AbsoluteSourcePath(source_path.clone()));
        }
    }
    Ok(())
}

fn validate_locations(chunk: &Chunk, source_path_count: usize) -> Result<(), ImageError> {
    for (instruction, location) in chunk.locations().iter().enumerate() {
        validate_location(instruction, location.line, location.column)?;
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

pub(super) fn validate_location(
    instruction: usize,
    line: u32,
    column: u32,
) -> Result<(), ImageError> {
    if line == 0 || column == 0 {
        return Err(ImageError::InvalidLocation {
            instruction,
            line,
            column,
        });
    }
    Ok(())
}

fn is_portably_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/')
        || path.starts_with('\\')
        || matches!(
            bytes,
            [drive, b':', separator, ..]
                if drive.is_ascii_alphabetic() && matches!(separator, b'/' | b'\\')
        )
}

fn validate_resources(image: &ProgramImage) -> Result<(), ImageError> {
    check_resource_size("instructions", image.chunk.len(), MAX_INSTRUCTIONS)?;
    check_resource_size("locations", image.chunk.locations().len(), MAX_LOCATIONS)?;
    check_resource_size("functions", image.chunk.functions().len(), MAX_FUNCTIONS)?;
    check_resource_size("constants", image.chunk.constants().len(), MAX_CONSTANTS)?;

    let mut string_bytes = 0;
    add_string_bytes(&mut string_bytes, image.identity.compiler_version.len())?;
    for unit in &image.identity.units {
        add_string_bytes(&mut string_bytes, unit.unit_name.len())?;
    }
    for path in &image.source_paths {
        add_string_bytes(&mut string_bytes, path.len())?;
    }
    for name in image.chunk.functions().keys() {
        add_string_bytes(&mut string_bytes, name.len())?;
    }
    for value in image.chunk.constants() {
        let persistent = PersistentValue::from_value(value).map_err(ImageError::PersistentValue)?;
        add_string_bytes(&mut string_bytes, persistent_string_bytes(&persistent))?;
    }
    check_resource_size("strings", string_bytes, MAX_TOTAL_STRING_BYTES)
}
