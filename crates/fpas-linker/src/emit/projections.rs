//! Relocation of source-checked projection types and their nominal names.

use super::strings::StringInterner;
use crate::{LinkError, plan::DebugTypeIds};
use fpas_code::{
    DebugProjection, DebugProjectionKind, DebugType, DebugTypeId, Executable, RecordLayout,
};
use fpas_unit::object::{ObjectDebugProjection, ObjectDebugProjectionKind};

pub(super) fn merge(
    projection: &ObjectDebugProjection,
    object: usize,
    types: &DebugTypeIds,
    strings: &mut StringInterner,
) -> Result<DebugProjection, LinkError> {
    Ok(DebugProjection {
        ty: types.translate(object, projection.ty)?,
        type_name: projection
            .type_name
            .as_deref()
            .map(|name| strings.intern(name))
            .transpose()?,
        kind: match &projection.kind {
            ObjectDebugProjectionKind::Unsupported => DebugProjectionKind::Unsupported,
            ObjectDebugProjectionKind::Operational => DebugProjectionKind::Operational,
            ObjectDebugProjectionKind::Scalar => DebugProjectionKind::Scalar,
            ObjectDebugProjectionKind::Array(child) => {
                DebugProjectionKind::Array(Box::new(merge(child, object, types, strings)?))
            }
            ObjectDebugProjectionKind::Reference(child) => {
                DebugProjectionKind::Reference(Box::new(merge(child, object, types, strings)?))
            }
            ObjectDebugProjectionKind::Record(fields) => DebugProjectionKind::Record(
                fields
                    .iter()
                    .map(|field| merge(field, object, types, strings))
                    .collect::<Result<_, _>>()?,
            ),
        },
    })
}

/// Imported layouts may erase fields whose defining layout is more precise.
/// Rebase erased terminal and array/reference storage types only when the
/// defining layout proves their precision. Explicit source refinements stay intact.
pub(super) fn rebase_canonical_terminals(image: &mut Executable) {
    for global in &mut image.globals {
        if let Some(projection) = &mut global.projection {
            rebase(&image.debug_types, &image.records, projection, global.ty);
        }
    }
    for function in &mut image.functions {
        for binding in &mut function.debug.bindings {
            if let Some(projection) = &mut binding.projection {
                rebase(&image.debug_types, &image.records, projection, binding.ty);
            }
        }
    }
}

fn rebase(
    types: &[DebugType],
    records: &[RecordLayout],
    node: &mut DebugProjection,
    expected: DebugTypeId,
) {
    if matches!(
        node.kind,
        DebugProjectionKind::Unsupported
            | DebugProjectionKind::Operational
            | DebugProjectionKind::Array(_)
            | DebugProjectionKind::Reference(_)
    ) && erased_matches(
        types,
        node.ty,
        expected,
        0,
        &mut std::collections::HashSet::new(),
    ) {
        node.ty = expected;
    }
    match &mut node.kind {
        DebugProjectionKind::Unsupported | DebugProjectionKind::Operational => {}
        DebugProjectionKind::Record(fields) => {
            let Some(DebugType::Record(record)) = types.get(node.ty.get() as usize) else {
                return;
            };
            let Some(layout) = records.get(record.get() as usize) else {
                return;
            };
            for (field, stored) in fields.iter_mut().zip(&layout.fields) {
                rebase(types, records, field, stored.ty);
            }
        }
        DebugProjectionKind::Array(child) => {
            let element = match types.get(expected.get() as usize) {
                Some(DebugType::Array(element)) => *element,
                _ => match types.get(node.ty.get() as usize) {
                    Some(DebugType::Array(element)) => *element,
                    _ => return,
                },
            };
            rebase(types, records, child, element);
        }
        DebugProjectionKind::Reference(child) => {
            let element = match types.get(expected.get() as usize) {
                Some(DebugType::Reference(element)) => *element,
                _ => match types.get(node.ty.get() as usize) {
                    Some(DebugType::Reference(element)) => *element,
                    _ => return,
                },
            };
            rebase(types, records, child, element);
        }
        DebugProjectionKind::Scalar => {}
    }
}

fn erased_matches(
    types: &[DebugType],
    erased: DebugTypeId,
    expected: DebugTypeId,
    depth: usize,
    seen: &mut std::collections::HashSet<(DebugTypeId, DebugTypeId)>,
) -> bool {
    if depth > 64 {
        return false;
    }
    let Some(left) = types.get(erased.get() as usize) else {
        return false;
    };
    let Some(right) = types.get(expected.get() as usize) else {
        return false;
    };
    if erased == expected || !seen.insert((erased, expected)) {
        return true;
    }
    let mut child = |left, right| erased_matches(types, left, right, depth + 1, seen);
    match (left, right) {
        (DebugType::Dynamic, _) => true,
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
        _ => left == right,
    }
}
