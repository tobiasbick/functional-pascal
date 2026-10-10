//! Stable scalar IR ids and type erasure for generic parameters.
//! See `docs/pascal/language/types/generics.md`.

use crate::{CompileError, error::internal_compiler_error};
use fpas_ir::{IrType, TypeDefinition, TypeId};
use fpas_sema::Ty;

/// Procedure result type.
pub(in crate::lowering) const UNIT: TypeId = TypeId::new(0);
/// Boolean value type.
pub(in crate::lowering) const BOOLEAN: TypeId = TypeId::new(1);
/// Signed integer value type.
pub(in crate::lowering) const INTEGER: TypeId = TypeId::new(2);
/// Real value type.
pub(in crate::lowering) const REAL: TypeId = TypeId::new(3);
/// String value type.
pub(in crate::lowering) const STRING: TypeId = TypeId::new(4);
/// Erased generic or unresolved value type.
pub(in crate::lowering) const DYNAMIC: TypeId = TypeId::new(5);

/// Initial type table shared by every lowered program and unit.
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

/// Lower a scalar or erased value that requires no aggregate layout.
pub(super) fn lower(ty: &Ty, line: u32, column: u32) -> Result<TypeId, CompileError> {
    match ty {
        Ty::Unit => Ok(UNIT),
        Ty::Boolean => Ok(BOOLEAN),
        Ty::Integer => Ok(INTEGER),
        Ty::Real => Ok(REAL),
        Ty::String => Ok(STRING),
        Ty::GenericParam(..) => Ok(DYNAMIC),
        Ty::Enum(enumeration) if !enumeration.has_data() => Ok(INTEGER),
        Ty::Distinct(distinct) => lower(&distinct.underlying, line, column),
        Ty::Error | Ty::Named(_) => Ok(DYNAMIC),
        other => Err(internal_compiler_error(
            format!("The compiler could not lower type `{other}`."),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            line,
            column,
        )),
    }
}

fn definition(id: TypeId, kind: IrType) -> TypeDefinition {
    TypeDefinition { id, kind }
}
