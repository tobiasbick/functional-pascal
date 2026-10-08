//! Typed construction stages supplied fields before omitted defaults.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use fpas_ir::{Operation, TypeId, ValueId};
use fpas_lexer::Span;
use fpas_parser::Expr;

use super::super::context::{LoweringContext, unsupported};
use crate::CompileError;

impl LoweringContext {
    /// Evaluate named fields once in written order and assemble declaration-order storage.
    pub(in crate::lowering) fn lower_record_construction(
        &mut self,
        arguments: &[Expr],
        ty: TypeId,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let layout = self
            .record_layout_id(ty)
            .ok_or_else(|| unsupported(span, "record construction layout"))?;
        let fields = self
            .record_fields(layout)
            .ok_or_else(|| unsupported(span, "record construction fields"))?;
        let name = self
            .record_layout_name(layout)
            .ok_or_else(|| unsupported(span, "record construction name"))?;
        let defaults = self.record_defaults.get(name).cloned().unwrap_or_default();
        let mut staged = vec![None; fields.len()];
        for argument in arguments {
            let name = argument
                .argument_name()
                .ok_or_else(|| unsupported(argument.span(), "unnamed record field"))?;
            let index = fields
                .iter()
                .position(|(field, _)| field.eq_ignore_ascii_case(name))
                .ok_or_else(|| unsupported(argument.span(), "unknown record field"))?;
            let expression = argument.argument_value();
            let field_ty = fields[index].1;
            let value = self.lower_expression_as(expression, field_ty)?;
            let local = self.declare_hidden_local(field_ty, expression.span())?;
            self.write_local(local, value, expression.span())?;
            staged[index] = Some(local);
        }
        for (index, (name, field_ty)) in fields.iter().enumerate() {
            if staged[index].is_some() {
                continue;
            }
            let expression = defaults
                .iter()
                .find(|(field, _)| field.eq_ignore_ascii_case(name))
                .and_then(|(_, expression)| expression.as_deref())
                .ok_or_else(|| unsupported(span, "missing record field default"))?;
            let value = self.lower_record_default(expression, *field_ty)?;
            let local = self.declare_hidden_local(*field_ty, expression.span())?;
            self.write_local(local, value, expression.span())?;
            staged[index] = Some(local);
        }
        let values = staged
            .into_iter()
            .zip(fields)
            .map(|(local, (_, ty))| {
                let local = local.ok_or_else(|| unsupported(span, "unstaged record field"))?;
                self.emit_value(Operation::ReadLocal(local), ty, span)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.emit_value(
            Operation::MakeRecord {
                layout,
                fields: values,
            },
            ty,
            span,
        )
    }
}
