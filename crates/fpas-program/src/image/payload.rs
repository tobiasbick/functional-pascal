//! JSON payload conversion for executable program images.

use std::collections::BTreeMap;

use fpas_bytecode::{Chunk, ExecutableError, Op, PersistentValue, SourceLocation};
use serde::{Deserialize, Serialize};

use super::resources::decode_encoded_chunk;
use super::{ImageError, ProgramImage, validate_location};
use crate::ProgramIdentity;

#[derive(Serialize)]
pub(super) struct EncodedChunk {
    pub(super) code: Vec<Op>,
    pub(super) constants: Vec<PersistentValue>,
    pub(super) locations: Vec<EncodedLocation>,
    pub(super) functions: BTreeMap<String, EncodedFunction>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EncodedLocation {
    pub(super) line: u32,
    pub(super) column: u32,
    pub(super) source_id: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EncodedFunction {
    pub(super) code_start: u32,
    pub(super) arity: u8,
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
    initial_string_bytes: usize,
) -> Result<ProgramImage, ImageError> {
    let payload = decode_encoded_chunk(bytes, initial_string_bytes)
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
        validate_location(instruction, location.line, location.column)?;
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
