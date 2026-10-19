//! Source projection encoding; operational register types and layouts stay unchanged.

use fpas_bytecode::{DebugProjection, DebugProjectionKind as Kind, DebugTypeId};

use super::metadata::MetadataBuilder;
use crate::CompileError;

pub(super) fn compile(
    projection: &fpas_ir::DebugProjection,
    metadata: &mut MetadataBuilder,
) -> Result<DebugProjection, CompileError> {
    use fpas_ir::DebugProjectionKind as Source;
    Ok(DebugProjection {
        ty: DebugTypeId::new(projection.ty.get()),
        type_name: projection
            .type_name
            .as_deref()
            .map(|name| metadata.intern_string(name))
            .transpose()?,
        kind: match &projection.kind {
            Source::Operational => Kind::Operational,
            Source::Unsupported => Kind::Unsupported,
            Source::Scalar => Kind::Scalar,
            Source::Array(element) => Kind::Array(Box::new(compile(element, metadata)?)),
            Source::Reference(inner) => Kind::Reference(Box::new(compile(inner, metadata)?)),
            Source::Record(fields) => Kind::Record(
                fields
                    .iter()
                    .map(|field| compile(field, metadata))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        },
    })
}
