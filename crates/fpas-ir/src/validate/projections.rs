//! Validate bounded source projections before recursively encoding debugger metadata.

use super::{ValidationError, ValidationErrorKind, program_error};
use crate::{DebugProjection, DebugProjectionKind as Kind, IrType, Program, TypeId};

fn invalid(reason: &'static str) -> ValidationError {
    program_error(ValidationErrorKind::DebugProjection { reason })
}

pub(super) fn validate(
    program: &Program,
    root: Option<&DebugProjection>,
    expected: TypeId,
) -> Result<(), ValidationError> {
    let Some(root) = root else {
        return Ok(());
    };
    let mut work = vec![(root, expected, 0)];
    let mut count = 0usize;
    let mut names = 0usize;
    while let Some((node, expected, depth)) = work.pop() {
        count += 1;
        if depth > 32 || count > 4096 {
            return Err(invalid("projection depth or node count exceeds its bound"));
        }
        if let Some(name) = &node.type_name {
            names = names.saturating_add(name.len());
            if name.is_empty() || name.len() > 4096 || names > 65536 {
                return Err(invalid("projection nominal name exceeds its bound"));
            }
        }
        if !compatible(program, expected, node.ty, 0) {
            return Err(invalid("projection contradicts operational type"));
        }
        let ty = &program
            .ty(node.ty)
            .ok_or_else(|| invalid("projection type is missing"))?
            .kind;
        match (&node.kind, ty) {
            (Kind::Operational, _) if node.ty == expected && node.type_name.is_none() => {}
            (Kind::Unsupported, _) => {}
            (Kind::Scalar, IrType::Integer | IrType::Real | IrType::String | IrType::Boolean) => {}
            (Kind::Scalar, IrType::Enum(id))
                if program.enum_layout(*id).is_some_and(|layout| {
                    layout
                        .variants
                        .iter()
                        .all(|variant| variant.fields.is_empty())
                }) => {}
            (Kind::Array(child), IrType::Array(inner))
            | (Kind::Reference(child), IrType::Reference(inner)) => {
                work.push((child, *inner, depth + 1))
            }
            (Kind::Record(fields), IrType::Record(id)) => {
                let layout = program
                    .record_layout(*id)
                    .ok_or_else(|| invalid("projection record is missing"))?;
                if fields.len() != layout.fields.len()
                    || fields.len() > 4096usize.saturating_sub(count)
                {
                    return Err(invalid(
                        "projection field count contradicts layout or node bound",
                    ));
                }
                work.extend(
                    fields
                        .iter()
                        .zip(&layout.fields)
                        .map(|(child, field)| (child, field.ty, depth + 1)),
                );
            }
            _ => return Err(invalid("projection shape contradicts its concrete type")),
        }
        if count.saturating_add(work.len()) > 4096 {
            return Err(invalid("projection node count exceeds its bound"));
        }
    }
    Ok(())
}

fn compatible(program: &Program, expected: TypeId, concrete: TypeId, depth: usize) -> bool {
    if depth > 64 {
        return false;
    }
    let Some(expected) = program.ty(expected) else {
        return false;
    };
    let Some(concrete) = program.ty(concrete) else {
        return false;
    };
    match (&expected.kind, &concrete.kind) {
        (IrType::Dynamic, _) => true,
        (IrType::Array(a), IrType::Array(b)) | (IrType::Reference(a), IrType::Reference(b)) => {
            compatible(program, *a, *b, depth + 1)
        }
        (a, b) => a == b,
    }
}
