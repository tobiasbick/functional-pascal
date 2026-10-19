//! Global binding metadata collected before root-function lowering.

use std::collections::BTreeMap;

use fpas_ir::{Global, GlobalId};
use fpas_parser::Decl;

use super::context::{self, GlobalBinding};
use super::types::TypeTable;
use crate::CompileError;

/// Collect globals and their lowering bindings from explicit declaration types.
pub(super) fn collect(
    declarations: &[Decl],
    type_table: &mut TypeTable,
) -> Result<(Vec<Global>, BTreeMap<String, GlobalBinding>), CompileError> {
    let mut globals = Vec::new();
    let mut bindings = BTreeMap::new();
    for declaration in declarations {
        let (name, type_expr, span, mutable) = match declaration {
            Decl::Const(definition) => (
                &definition.name,
                &definition.type_expr,
                definition.span,
                false,
            ),
            Decl::Var(definition) => (
                &definition.name,
                &definition.type_expr,
                definition.span,
                true,
            ),
            _ => continue,
        };
        let ty = type_table.type_expr(type_expr)?;
        let id = GlobalId::try_from_index(globals.len())
            .map_err(|_| context::unsupported(span, "global identifier overflow"))?;
        globals.push(Global {
            id,
            name: name.clone(),
            ty,
            type_name: type_table.declared_scalar_name(type_expr),
            projection: type_table.debug_projection(
                span.diagnostic_span_or_synthetic(),
                name,
                ty,
            )?,
            mutable,
            initializer: None,
        });
        bindings.insert(name.to_ascii_lowercase(), GlobalBinding { id, ty });
    }
    Ok((globals, bindings))
}
