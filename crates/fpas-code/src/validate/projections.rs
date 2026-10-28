//! Bounds and operational compatibility for owned source-checked projections.

use super::{ValidationError, ValidationErrorKind};
use crate::{DebugProjection, DebugProjectionKind, DebugType, DebugTypeId, Executable};

fn invalid(reason: &'static str) -> ValidationError {
    ValidationError::executable(ValidationErrorKind::DebugProjection { reason })
}

pub(super) fn validate(
    image: &Executable,
    root: Option<&DebugProjection>,
    operational: DebugTypeId,
) -> Result<(), ValidationError> {
    let Some(root) = root else {
        return Ok(());
    };
    let mut pending = vec![(root, operational, 0)];
    let mut nodes = 0usize;
    let mut names = 0usize;
    while let Some((node, expected, depth)) = pending.pop() {
        nodes += 1;
        if depth > crate::MAX_DEBUG_PROJECTION_DEPTH || nodes > crate::MAX_DEBUG_PROJECTION_NODES {
            return Err(invalid("projection depth or node count exceeds its limit"));
        }
        if let Some(name) = node.type_name {
            let name = image
                .strings
                .get(name)
                .ok_or_else(|| invalid("invalid projection type name"))?;
            names = names.saturating_add(name.len());
            if name.is_empty() || names > crate::MAX_DEBUG_PROJECTION_NAME_BYTES {
                return Err(invalid("projection nominal names exceed their limit"));
            }
        }
        if !compatible(
            image,
            expected,
            node.ty,
            !matches!(node.kind, DebugProjectionKind::Operational),
        ) {
            return Err(invalid(
                "projection type disagrees with operational storage",
            ));
        }
        let ty = image
            .debug_types
            .get(node.ty.get() as usize)
            .ok_or_else(|| invalid("missing concrete projection type"))?;
        match (&node.kind, ty) {
            (DebugProjectionKind::Unsupported, _) => {}
            (DebugProjectionKind::Operational, _) if node.type_name.is_none() => {}
            (
                DebugProjectionKind::Scalar,
                DebugType::Integer | DebugType::Real | DebugType::Boolean | DebugType::String,
            ) => {}
            (DebugProjectionKind::Scalar, DebugType::Enum(id))
                if !image
                    .enum_variants
                    .iter()
                    .any(|variant| variant.owner == *id && !variant.fields.is_empty()) => {}
            (DebugProjectionKind::Array(child), DebugType::Array(element))
            | (DebugProjectionKind::Reference(child), DebugType::Reference(element)) => {
                pending.push((child, *element, depth + 1))
            }
            (DebugProjectionKind::Record(fields), DebugType::Record(id)) => {
                let layout = image
                    .records
                    .get(id.get() as usize)
                    .ok_or_else(|| invalid("missing projection record layout"))?;
                if fields.len() != layout.fields.len()
                    || fields.len() > crate::MAX_DEBUG_PROJECTION_NODES.saturating_sub(nodes)
                {
                    return Err(invalid(
                        "projection record field shape exceeds or disagrees with layout",
                    ));
                }
                pending.extend(
                    fields
                        .iter()
                        .zip(&layout.fields)
                        .map(|(field, stored)| (field, stored.ty, depth + 1)),
                );
            }
            _ => {
                return Err(invalid(
                    "projection structure disagrees with its concrete type",
                ));
            }
        }
        if pending.len().saturating_add(nodes) > crate::MAX_DEBUG_PROJECTION_NODES {
            return Err(invalid("projection node count exceeds its limit"));
        }
    }
    Ok(())
}

fn compatible(
    image: &Executable,
    expected: DebugTypeId,
    concrete: DebugTypeId,
    allow_refinement: bool,
) -> bool {
    compatible_inner(
        image,
        expected,
        concrete,
        allow_refinement,
        0,
        &mut std::collections::HashSet::new(),
    )
}

fn compatible_inner(
    image: &Executable,
    expected: DebugTypeId,
    concrete: DebugTypeId,
    allow_refinement: bool,
    depth: usize,
    visited: &mut std::collections::HashSet<(u32, u32)>,
) -> bool {
    if depth > 64 {
        return false;
    }
    let Some(expected_type) = image.debug_types.get(expected.get() as usize) else {
        return false;
    };
    let Some(concrete_type) = image.debug_types.get(concrete.get() as usize) else {
        return false;
    };
    if expected == concrete || !visited.insert((expected.get(), concrete.get())) {
        return true;
    }
    let mut child =
        |left, right| compatible_inner(image, left, right, allow_refinement, depth + 1, visited);
    match (expected_type, concrete_type) {
        (DebugType::Dynamic, _) if allow_refinement => true,
        (DebugType::Array(left), DebugType::Array(right))
        | (DebugType::Reference(left), DebugType::Reference(right))
        | (DebugType::Option(left), DebugType::Option(right))
        | (DebugType::Cell(left), DebugType::Cell(right))
        | (DebugType::Task(left), DebugType::Task(right))
        | (DebugType::Channel(left), DebugType::Channel(right)) => child(*left, *right),
        (
            DebugType::Dictionary {
                key: left_key,
                value: left_value,
            },
            DebugType::Dictionary {
                key: right_key,
                value: right_value,
            },
        )
        | (
            DebugType::Result {
                ok: left_key,
                error: left_value,
            },
            DebugType::Result {
                ok: right_key,
                error: right_value,
            },
        ) => child(*left_key, *right_key) && child(*left_value, *right_value),
        (
            DebugType::Function {
                parameters: left,
                result: left_result,
            },
            DebugType::Function {
                parameters: right,
                result: right_result,
            },
        ) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| child(*left, *right))
                && child(*left_result, *right_result)
        }
        _ => expected_type == concrete_type,
    }
}
