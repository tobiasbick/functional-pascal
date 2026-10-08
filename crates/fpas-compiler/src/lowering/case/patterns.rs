//! Recursive pattern tests for Result, Option, and enum case labels.
//!
//! Each test branches to the label's failure block on a mismatch and leaves
//! lowering in the success block. Matched payloads are stored in hidden locals,
//! so nested tests and bindings read them after earlier branches.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use fpas_ir::{BinaryOperation, BlockId, IrType, LocalId, Operation, Terminator, TypeId};
use fpas_parser::{DesignatorPart, DestructureVariant, Expr, Pattern};

use crate::CompileError;

use super::super::context::{LoweringContext, target, unsupported};
use super::super::types;

/// Bound name, its type, and the hidden local holding the matched value.
pub(in crate::lowering) type PatternBinding = (String, TypeId, LocalId);

impl LoweringContext {
    /// Emits the tests of `pattern` against the value stored in `source`.
    pub(in crate::lowering) fn lower_pattern_test(
        &mut self,
        source: LocalId,
        ty: TypeId,
        pattern: &Pattern,
        fail: BlockId,
        bindings: &mut Vec<PatternBinding>,
    ) -> Result<(), CompileError> {
        match pattern {
            Pattern::Binding { name, .. } => {
                bindings.push((name.clone(), ty, source));
                Ok(())
            }
            Pattern::Wildcard(_) => Ok(()),
            Pattern::Value(expr) => self.lower_value_test(source, ty, expr, fail),
            Pattern::Variant {
                constructor,
                fields,
                span,
            } => {
                let Some(IrType::Enum(layout)) = self.type_kind(ty) else {
                    return Err(unsupported(*span, "variant pattern type"));
                };
                let name = match constructor.parts.last() {
                    Some(DesignatorPart::Ident(name, _)) => name.as_str(),
                    _ => return Err(unsupported(*span, "variant pattern name")),
                };
                let (variant, field_types) = self
                    .enum_variant(layout, name)
                    .ok_or_else(|| unsupported(*span, "variant pattern variant"))?;
                let value = self.emit_value(Operation::ReadLocal(source), ty, *span)?;
                let matched = self.emit_value(
                    Operation::TestVariant {
                        value,
                        layout,
                        variant,
                    },
                    types::BOOLEAN,
                    *span,
                )?;
                self.continue_if(matched, fail, *span)?;
                for (index, field) in fields.iter().enumerate() {
                    if matches!(field.pattern, Pattern::Wildcard(_)) {
                        continue;
                    }
                    let field_ty = field_types
                        .get(index)
                        .copied()
                        .ok_or_else(|| unsupported(*span, "variant pattern field"))?;
                    let field_id = fpas_ir::FieldId::try_from_index(index)
                        .map_err(|_| unsupported(*span, "variant pattern field"))?;
                    let value = self.emit_value(Operation::ReadLocal(source), ty, *span)?;
                    let payload = self.emit_value(
                        Operation::LoadEnumField {
                            value,
                            layout,
                            variant,
                            field: field_id,
                        },
                        field_ty,
                        *span,
                    )?;
                    let local = self.store_matched(payload, field_ty, *span)?;
                    self.lower_pattern_test(local, field_ty, &field.pattern, fail, bindings)?;
                }
                Ok(())
            }
            Pattern::Destructure {
                variant,
                payload,
                span,
            } => {
                let value = self.emit_value(Operation::ReadLocal(source), ty, *span)?;
                let (test, negate, payload_ty) = match (self.type_kind(ty), variant) {
                    (Some(IrType::Result { ok, .. }), DestructureVariant::Ok) => {
                        (Operation::IsResultOk(value), false, ok)
                    }
                    (Some(IrType::Result { error, .. }), DestructureVariant::Error) => {
                        (Operation::IsResultOk(value), true, error)
                    }
                    (Some(IrType::Option(inner)), DestructureVariant::Some) => {
                        (Operation::IsOptionSome(value), false, inner)
                    }
                    (Some(IrType::Option(inner)), DestructureVariant::None) => {
                        (Operation::IsOptionSome(value), true, inner)
                    }
                    _ => return Err(unsupported(*span, "Result or Option pattern")),
                };
                let mut matched = self.emit_value(test, types::BOOLEAN, *span)?;
                if negate {
                    matched = self.emit_value(
                        Operation::Unary {
                            operation: fpas_ir::UnaryOperation::NotBoolean,
                            operand: matched,
                        },
                        types::BOOLEAN,
                        *span,
                    )?;
                }
                self.continue_if(matched, fail, *span)?;
                let Some(payload) = payload else {
                    return Ok(());
                };
                if matches!(**payload, Pattern::Wildcard(_)) {
                    return Ok(());
                }
                let value = self.emit_value(Operation::ReadLocal(source), ty, *span)?;
                let unwrap = match variant {
                    DestructureVariant::Ok => Operation::UnwrapOk(value),
                    DestructureVariant::Error => Operation::UnwrapError(value),
                    _ => Operation::UnwrapSome(value),
                };
                let inner = self.emit_value(unwrap, payload_ty, *span)?;
                let local = self.store_matched(inner, payload_ty, *span)?;
                self.lower_pattern_test(local, payload_ty, payload, fail, bindings)
            }
        }
    }

    /// Tests a fieldless variant, or compares the value with a constant.
    pub(super) fn lower_value_test(
        &mut self,
        source: LocalId,
        ty: TypeId,
        expr: &Expr,
        fail: BlockId,
    ) -> Result<(), CompileError> {
        let span = expr.span();
        if let (Some(IrType::Enum(layout)), Expr::Designator(designator)) =
            (self.type_kind(ty), expr)
            && let Some(DesignatorPart::Ident(name, _)) = designator.parts.last()
            && let Some((variant, _)) = self.enum_variant(layout, name)
        {
            let value = self.emit_value(Operation::ReadLocal(source), ty, span)?;
            let matched = self.emit_value(
                Operation::TestVariant {
                    value,
                    layout,
                    variant,
                },
                types::BOOLEAN,
                span,
            )?;
            return self.continue_if(matched, fail, span);
        }
        let expected = self.lower_expression_as(expr, ty)?;
        let expected = self.save_value(expected);
        let value = self.emit_value(Operation::ReadLocal(source), ty, span)?;
        let expected = self.restore_value(expected, span)?;
        let matched = self.emit_binary(
            BinaryOperation::Equal,
            value,
            expected,
            types::BOOLEAN,
            span,
        )?;
        self.continue_if(matched, fail, span)
    }

    /// Continues in a new block when `condition` holds and branches to `fail` otherwise.
    pub(in crate::lowering) fn continue_if(
        &mut self,
        condition: fpas_ir::ValueId,
        fail: BlockId,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let matched = self.new_block(span)?;
        self.terminate(Terminator::Branch {
            condition,
            then_target: target(matched),
            else_target: target(fail),
        })?;
        self.switch_to(matched);
        Ok(())
    }

    fn store_matched(
        &mut self,
        value: fpas_ir::ValueId,
        ty: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<LocalId, CompileError> {
        let local = self.declare_hidden_local(ty, span)?;
        self.write_local(local, value, span)?;
        Ok(local)
    }
}
