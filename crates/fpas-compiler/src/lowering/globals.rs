//! Global binding metadata collected before root-function lowering.

use std::collections::BTreeMap;

use fpas_ir::{Global, GlobalId};
use fpas_parser::Decl;

use super::context::{self, GlobalBinding};
use super::types::TypeTable;
use crate::CompileError;

/// Collect globals and their lowering bindings with precise task result metadata.
pub(super) fn collect(
    declarations: &[Decl],
    metadata: &fpas_sema::AnalysisMetadata,
    type_table: &mut TypeTable,
) -> Result<(Vec<Global>, BTreeMap<String, GlobalBinding>), CompileError> {
    let mut globals = Vec::new();
    let mut bindings = BTreeMap::new();
    for declaration in declarations {
        let (name, type_expr, value, span, mutable) = match declaration {
            Decl::Const(definition) => (
                &definition.name,
                &definition.type_expr,
                &definition.value,
                definition.span,
                false,
            ),
            Decl::Var(definition) => (
                &definition.name,
                &definition.type_expr,
                &definition.value,
                definition.span,
                false,
            ),
            Decl::MutableVar(definition) => (
                &definition.name,
                &definition.type_expr,
                &definition.value,
                definition.span,
                true,
            ),
            _ => continue,
        };
        let declared = type_table.type_expr(type_expr)?;
        let ty = type_table.specialize_task_binding_from_sema(
            declared,
            metadata.expr_types.get(&fpas_sema::expr_lookup_key(value)),
            span.line,
            span.column,
        )?;
        let id = GlobalId::try_from_index(globals.len())
            .map_err(|_| context::unsupported(span, "global identifier overflow"))?;
        globals.push(Global {
            id,
            name: name.clone(),
            ty,
            mutable,
            initializer: None,
        });
        bindings.insert(name.to_ascii_lowercase(), GlobalBinding { id, ty });
    }
    Ok((globals, bindings))
}
