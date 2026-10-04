//! Declaration-scoped evaluation of record field defaults.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::{
    closures::ClosureRegistry,
    context::{Callable, FunctionInput, LoweringContext, unsupported},
    types::TypeTable,
};
use crate::CompileError;
use fpas_ir::{Function, FunctionId, Terminator, TypeId};
use fpas_parser::{Decl, Expr, TypeBody};
use fpas_sema::{AnalysisMetadata, Ty};
use std::collections::BTreeMap;

/// One original default expression and its ordinary implementation signature.
pub(super) struct Initializer<'a> {
    pub id: FunctionId,
    pub name: String,
    pub expression: &'a Expr,
    pub result: TypeId,
}

/// Allocate internal calls before imports and anonymous closures.
pub(super) fn collect<'a>(
    declarations: &'a [Decl],
    metadata: &AnalysisMetadata,
    types: &mut TypeTable,
    callables: &mut BTreeMap<String, Callable>,
    first: u32,
) -> Result<Vec<Initializer<'a>>, CompileError> {
    let mut initializers = Vec::new();
    for declaration in declarations {
        let Decl::TypeDef(definition) = declaration else {
            continue;
        };
        let TypeBody::Record(record) = &definition.body else {
            continue;
        };
        let Some(Ty::Record(ty)) = metadata
            .named_types
            .get(&definition.name.to_ascii_lowercase())
        else {
            return Err(unsupported(
                definition.span,
                "default record declaration type",
            ));
        };
        for field in &record.fields {
            let Some(expression) = field.default_value.as_deref() else {
                continue;
            };
            let field_type = ty
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                .map(|(_, ty)| ty)
                .ok_or_else(|| unsupported(field.span, "default field type"))?;
            let result = types.intern(field_type, field.span.line, field.span.column)?;
            let name = fpas_unit::interface::record_default_initializer(
                &ty.name,
                ty.owner_unit.as_deref(),
                &field.name,
            );
            let index = u32::try_from(initializers.len())
                .map_err(|_| unsupported(field.span, "default initializer overflow"))?;
            let id = FunctionId::new(
                first
                    .checked_add(index)
                    .ok_or_else(|| unsupported(field.span, "default initializer overflow"))?,
            );
            callables.insert(
                name.clone(),
                Callable {
                    function: id,
                    parameters: Vec::new(),
                    result,
                    value_type: types.function_type(Vec::new(), result, field.span)?,
                    captures: Vec::new(),
                },
            );
            initializers.push(Initializer {
                id,
                name,
                expression,
                result,
            });
        }
    }
    Ok(initializers)
}

/// Discover captures relative to the initializer rather than its eventual caller.
pub(super) fn discover<'a>(
    initializers: &[Initializer<'a>],
    closures: &mut ClosureRegistry<'a>,
    metadata: &AnalysisMetadata,
    types: &mut TypeTable,
) -> Result<(), CompileError> {
    for initializer in initializers {
        closures.discover_expression(initializer.expression, initializer.id, metadata, types)?;
    }
    Ok(())
}

/// Lower an internal default without granting source-level pure callable capability.
pub(super) fn lower(
    initializer: &Initializer<'_>,
    input: FunctionInput<'_>,
) -> Result<(Function, TypeTable), CompileError> {
    let mut context = LoweringContext::new(input)?;
    let value = context.lower_expression_as(initializer.expression, initializer.result)?;
    context.terminate(Terminator::Return(Some(value)))?;
    context.finish(initializer.expression.span())
}
