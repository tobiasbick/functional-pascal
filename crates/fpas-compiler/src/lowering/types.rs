//! Compact semantic-to-IR scalar type mapping.

use fpas_ir::{IrType, TypeDefinition, TypeId};
use fpas_sema::Ty;

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) const UNIT: TypeId = TypeId::new(0);
pub(super) const BOOLEAN: TypeId = TypeId::new(1);
pub(super) const INTEGER: TypeId = TypeId::new(2);
pub(super) const REAL: TypeId = TypeId::new(3);
pub(super) const STRING: TypeId = TypeId::new(4);
pub(super) const DYNAMIC: TypeId = TypeId::new(5);

pub(super) fn scalar_type_table() -> Vec<TypeDefinition> {
    vec![
        definition(UNIT, IrType::Unit),
        definition(BOOLEAN, IrType::Boolean),
        definition(INTEGER, IrType::Integer),
        definition(REAL, IrType::Real),
        definition(STRING, IrType::String),
        definition(DYNAMIC, IrType::Dynamic),
    ]
}

pub(super) fn lower(ty: &Ty, line: u32, column: u32) -> Result<TypeId, CompileError> {
    match ty {
        Ty::Unit => Ok(UNIT),
        Ty::Boolean => Ok(BOOLEAN),
        Ty::Integer => Ok(INTEGER),
        Ty::Real => Ok(REAL),
        Ty::String => Ok(STRING),
        Ty::GenericParam(..) => Ok(DYNAMIC),
        other => Err(internal_compiler_error(
            format!("Type `{other}` is outside the P3 scalar register subset."),
            "Use only integer, real, boolean, string, and Unit values in this development path.",
            line,
            column,
        )),
    }
}

fn definition(id: TypeId, kind: IrType) -> TypeDefinition {
    TypeDefinition { id, kind }
}
