//! Conversion and bounded validation for source-checked projection trees.

use super::{
    ObjectDebugProjection, ObjectDebugProjectionKind, ObjectDebugType, ObjectError,
    RelocatableObject,
};

pub(super) fn from_executable(
    node: &fpas_code::DebugProjection,
    strings: &impl Fn(fpas_code::StringId) -> Result<String, ObjectError>,
) -> Result<ObjectDebugProjection, ObjectError> {
    use fpas_code::DebugProjectionKind as Source;
    Ok(ObjectDebugProjection {
        ty: node.ty.get(),
        type_name: node.type_name.map(strings).transpose()?,
        kind: match &node.kind {
            Source::Unsupported => ObjectDebugProjectionKind::Unsupported,
            Source::Operational => ObjectDebugProjectionKind::Operational,
            Source::Scalar => ObjectDebugProjectionKind::Scalar,
            Source::Array(child) => {
                ObjectDebugProjectionKind::Array(Box::new(from_executable(child, strings)?))
            }
            Source::Reference(child) => {
                ObjectDebugProjectionKind::Reference(Box::new(from_executable(child, strings)?))
            }
            Source::Record(fields) => ObjectDebugProjectionKind::Record(
                fields
                    .iter()
                    .map(|field| from_executable(field, strings))
                    .collect::<Result<_, _>>()?,
            ),
        },
    })
}

pub(super) fn validate(image: &RelocatableObject) -> Result<(), ObjectError> {
    for global in &image.globals {
        validate_root(image, global.projection.as_ref(), global.ty)?;
    }
    for function in &image.functions {
        for binding in &function.debug.bindings {
            validate_root(image, binding.projection.as_ref(), binding.ty)?;
        }
    }
    Ok(())
}

fn invalid() -> ObjectError {
    ObjectError::InvalidTableReference("source-checked debug projection")
}

fn validate_root(
    image: &RelocatableObject,
    root: Option<&ObjectDebugProjection>,
    expected: u32,
) -> Result<(), ObjectError> {
    let Some(root) = root else {
        return Ok(());
    };
    let mut pending = vec![(root, expected, 0)];
    let mut nodes = 0usize;
    let mut names = 0usize;
    while let Some((node, expected, depth)) = pending.pop() {
        nodes += 1;
        if depth > fpas_code::MAX_DEBUG_PROJECTION_DEPTH
            || nodes > fpas_code::MAX_DEBUG_PROJECTION_NODES
        {
            return Err(invalid());
        }
        if let Some(name) = &node.type_name {
            names = names.saturating_add(name.len());
            if name.is_empty() || names > fpas_code::MAX_DEBUG_PROJECTION_NAME_BYTES {
                return Err(invalid());
            }
        }
        if !compatible(
            image,
            expected,
            node.ty,
            !matches!(node.kind, ObjectDebugProjectionKind::Operational),
        ) {
            return Err(invalid());
        }
        let ty = image
            .debug_types
            .get(node.ty as usize)
            .ok_or_else(invalid)?;
        match (&node.kind, ty) {
            (ObjectDebugProjectionKind::Unsupported, _) => {}
            (ObjectDebugProjectionKind::Operational, _) if node.type_name.is_none() => {}
            (
                ObjectDebugProjectionKind::Scalar,
                ObjectDebugType::Integer
                | ObjectDebugType::Real
                | ObjectDebugType::Boolean
                | ObjectDebugType::String,
            ) => {}
            (ObjectDebugProjectionKind::Scalar, ObjectDebugType::Enum(name)) => {
                if image
                    .enums
                    .iter()
                    .filter(|enumeration| enumeration.name.eq_ignore_ascii_case(name))
                    .any(|enumeration| {
                        enumeration
                            .variants
                            .iter()
                            .any(|variant| !variant.fields.is_empty())
                    })
                {
                    return Err(invalid());
                }
            }
            (ObjectDebugProjectionKind::Array(child), ObjectDebugType::Array(element))
            | (ObjectDebugProjectionKind::Reference(child), ObjectDebugType::Reference(element)) => {
                pending.push((child, *element, depth + 1))
            }
            (ObjectDebugProjectionKind::Record(fields), ObjectDebugType::Record(name)) => {
                if fields.len() > fpas_code::MAX_DEBUG_PROJECTION_NODES.saturating_sub(nodes) {
                    return Err(invalid());
                }
                let layout = image
                    .records
                    .iter()
                    .find(|record| record.name.eq_ignore_ascii_case(name));
                if let Some(layout) = layout {
                    if fields.len() != layout.field_types.len() {
                        return Err(invalid());
                    }
                    pending.extend(
                        fields
                            .iter()
                            .zip(&layout.field_types)
                            .map(|(field, stored)| (field, *stored, depth + 1)),
                    );
                } else {
                    // Imported symbolic layouts are checked fully by the final executable verifier.
                    pending.extend(fields.iter().map(|field| (field, field.ty, depth + 1)));
                }
            }
            _ => return Err(invalid()),
        }
        if pending.len().saturating_add(nodes) > fpas_code::MAX_DEBUG_PROJECTION_NODES {
            return Err(invalid());
        }
    }
    Ok(())
}

fn compatible(
    image: &RelocatableObject,
    expected: u32,
    concrete: u32,
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
    image: &RelocatableObject,
    expected: u32,
    concrete: u32,
    allow_refinement: bool,
    depth: usize,
    visited: &mut std::collections::HashSet<(u32, u32)>,
) -> bool {
    if depth > 64 {
        return false;
    }
    let Some(expected_type) = image.debug_types.get(expected as usize) else {
        return false;
    };
    let Some(concrete_type) = image.debug_types.get(concrete as usize) else {
        return false;
    };
    if expected == concrete || !visited.insert((expected, concrete)) {
        return true;
    }
    let mut child =
        |left, right| compatible_inner(image, left, right, allow_refinement, depth + 1, visited);
    match (expected_type, concrete_type) {
        (ObjectDebugType::Dynamic, _) if allow_refinement => true,
        (ObjectDebugType::Array(left), ObjectDebugType::Array(right))
        | (ObjectDebugType::Reference(left), ObjectDebugType::Reference(right))
        | (ObjectDebugType::Option(left), ObjectDebugType::Option(right))
        | (ObjectDebugType::Cell(left), ObjectDebugType::Cell(right))
        | (ObjectDebugType::Task(left), ObjectDebugType::Task(right))
        | (ObjectDebugType::Channel(left), ObjectDebugType::Channel(right)) => child(*left, *right),
        (
            ObjectDebugType::Dictionary {
                key: left_key,
                value: left_value,
            },
            ObjectDebugType::Dictionary {
                key: right_key,
                value: right_value,
            },
        )
        | (
            ObjectDebugType::Result {
                ok: left_key,
                error: left_value,
            },
            ObjectDebugType::Result {
                ok: right_key,
                error: right_value,
            },
        ) => child(*left_key, *right_key) && child(*left_value, *right_value),
        (
            ObjectDebugType::Function {
                parameters: left,
                result: left_result,
            },
            ObjectDebugType::Function {
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
        (ObjectDebugType::Record(left), ObjectDebugType::Record(right))
        | (ObjectDebugType::Enum(left), ObjectDebugType::Enum(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        _ => expected_type == concrete_type,
    }
}
