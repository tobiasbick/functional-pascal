//! Explicit bounded binary encoding of owned source-checked projection trees.

use super::sections::{SectionReader, write_u8, write_u32};
use super::{FormatError, check_limit};
use fpas_bytecode::{DebugProjection, DebugProjectionKind, DebugTypeId, StringId};

const UNSUPPORTED: u8 = 0;
const SCALAR: u8 = 1;
const ARRAY: u8 = 2;
const RECORD: u8 = 3;
const REFERENCE: u8 = 4;
const OPERATIONAL: u8 = 5;

pub(super) fn encode(
    output: &mut Vec<u8>,
    projection: Option<&DebugProjection>,
) -> Result<(), FormatError> {
    write_u8(output, u8::from(projection.is_some()));
    if let Some(projection) = projection {
        encode_node(output, projection, 0, &mut 0)?;
    }
    Ok(())
}

fn charge(depth: usize, nodes: &mut usize) -> Result<(), FormatError> {
    check_limit(
        "debug_projection_depth",
        depth,
        fpas_bytecode::MAX_DEBUG_PROJECTION_DEPTH,
    )?;
    *nodes += 1;
    check_limit(
        "debug_projection_nodes",
        *nodes,
        fpas_bytecode::MAX_DEBUG_PROJECTION_NODES,
    )
}

fn encode_node(
    output: &mut Vec<u8>,
    node: &DebugProjection,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), FormatError> {
    charge(depth, nodes)?;
    write_u32(output, node.ty.get());
    write_u32(output, node.type_name.map_or(u32::MAX, |name| name.get()));
    match &node.kind {
        DebugProjectionKind::Unsupported => write_u8(output, UNSUPPORTED),
        DebugProjectionKind::Operational => write_u8(output, OPERATIONAL),
        DebugProjectionKind::Scalar => write_u8(output, SCALAR),
        DebugProjectionKind::Array(child) | DebugProjectionKind::Reference(child) => {
            write_u8(
                output,
                if matches!(&node.kind, DebugProjectionKind::Array(_)) {
                    ARRAY
                } else {
                    REFERENCE
                },
            );
            encode_node(output, child, depth + 1, nodes)?;
        }
        DebugProjectionKind::Record(fields) => {
            check_limit(
                "debug_projection_nodes",
                nodes.saturating_add(fields.len()),
                fpas_bytecode::MAX_DEBUG_PROJECTION_NODES,
            )?;
            write_u8(output, RECORD);
            write_u32(output, fields.len() as u32);
            for field in fields {
                encode_node(output, field, depth + 1, nodes)?;
            }
        }
    }
    Ok(())
}

pub(super) fn decode(
    reader: &mut SectionReader<'_>,
) -> Result<Option<DebugProjection>, FormatError> {
    match reader.u8("debug_has_projection")? {
        0 => Ok(None),
        1 => decode_node(reader, 0, &mut 0).map(Some),
        value => Err(FormatError::InvalidValue {
            field: "debug_has_projection",
            value: value.into(),
        }),
    }
}

fn decode_node(
    reader: &mut SectionReader<'_>,
    depth: usize,
    nodes: &mut usize,
) -> Result<DebugProjection, FormatError> {
    charge(depth, nodes)?;
    let ty = DebugTypeId::new(reader.u32("debug_projection_type")?);
    let type_name = match reader.u32("debug_projection_type_name")? {
        u32::MAX => None,
        name => Some(StringId::new(name)),
    };
    let kind = match reader.u8("debug_projection_kind")? {
        UNSUPPORTED => DebugProjectionKind::Unsupported,
        OPERATIONAL => DebugProjectionKind::Operational,
        SCALAR => DebugProjectionKind::Scalar,
        ARRAY => DebugProjectionKind::Array(Box::new(decode_node(reader, depth + 1, nodes)?)),
        REFERENCE => {
            DebugProjectionKind::Reference(Box::new(decode_node(reader, depth + 1, nodes)?))
        }
        RECORD => {
            let count = reader.u32("debug_projection_field_count")? as usize;
            check_limit(
                "debug_projection_nodes",
                nodes.saturating_add(count),
                fpas_bytecode::MAX_DEBUG_PROJECTION_NODES,
            )?;
            reader.ensure_entries(count, 9, "debug_projection_field")?;
            // Charge each decoded child before growing the owned field vector.
            // Reserving every sibling here could multiply allocations through
            // deeply nested records before their total node budget is known.
            let mut fields = Vec::new();
            for _ in 0..count {
                fields.push(decode_node(reader, depth + 1, nodes)?);
            }
            DebugProjectionKind::Record(fields)
        }
        value => {
            return Err(FormatError::InvalidValue {
                field: "debug_projection_kind",
                value: value.into(),
            });
        }
    };
    Ok(DebugProjection {
        ty,
        type_name,
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(output: &mut Vec<u8>, kind: u8) {
        write_u32(output, 0);
        write_u32(output, u32::MAX);
        write_u8(output, kind);
    }

    #[test]
    fn excessive_children_are_rejected_before_reservation() {
        let mut bytes = vec![1];
        node(&mut bytes, RECORD);
        write_u32(&mut bytes, u32::MAX);
        let error =
            decode(&mut SectionReader::new(&bytes, "projection")).expect_err("bounded field count");
        assert!(matches!(
            error,
            FormatError::LimitExceeded {
                field: "debug_projection_nodes",
                ..
            }
        ));
    }

    #[test]
    fn excessive_depth_and_unknown_tags_are_rejected() {
        let mut bytes = vec![1];
        for _ in 0..33 {
            node(&mut bytes, ARRAY);
        }
        node(&mut bytes, UNSUPPORTED);
        assert!(matches!(
            decode(&mut SectionReader::new(&bytes, "projection")),
            Err(FormatError::LimitExceeded {
                field: "debug_projection_depth",
                ..
            })
        ));
        let mut unknown = vec![1];
        node(&mut unknown, 255);
        assert!(matches!(
            decode(&mut SectionReader::new(&unknown, "projection")),
            Err(FormatError::InvalidValue {
                field: "debug_projection_kind",
                ..
            })
        ));
    }

    #[test]
    fn operational_tail_has_its_own_explicit_round_trip_tag() {
        let projection = DebugProjection {
            ty: DebugTypeId::new(17),
            type_name: None,
            kind: DebugProjectionKind::Operational,
        };
        let mut bytes = Vec::new();
        encode(&mut bytes, Some(&projection)).expect("encode operational tail");
        assert_eq!(bytes.last().copied(), Some(OPERATIONAL));
        assert_eq!(
            decode(&mut SectionReader::new(&bytes, "projection")).expect("decode tail"),
            Some(projection)
        );
    }
}
