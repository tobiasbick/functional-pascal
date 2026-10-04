//! Short-circuit recursive pattern tests; payload reads follow discriminator tests.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use fpas_ir::{
    BinaryOperation, BlockId, Constant, FieldId, IrType, Operation, Terminator, TypeId,
    UnaryOperation, ValueId,
};
use fpas_lexer::Span;
use fpas_parser::Pattern;
use fpas_sema::{PatternVariant, Ty};

use super::super::context::{LoweringContext, target, unsupported};
use super::super::types;
use crate::CompileError;

impl LoweringContext {
    /// Reserve the bindings shared by grouped patterns before lowering their alternatives.
    pub(in crate::lowering) fn declare_pattern_bindings(
        &mut self,
        pattern: &Pattern,
    ) -> Result<(), CompileError> {
        match pattern {
            Pattern::Binding { name, span } => {
                let info = self
                    .pattern_infos
                    .get(&fpas_sema::pattern_lookup_key(pattern))
                    .ok_or_else(|| unsupported(*span, "pattern binding type"))?;
                let ty = self.type_table.id(&info.ty, span.line, span.column)?;
                self.declare_local(name, ty, false, *span)?;
            }
            Pattern::Variant { arguments, .. } => {
                for argument in arguments {
                    self.declare_pattern_bindings(argument)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Test a recursive pattern, branching to failure before any inactive payload read.
    pub(in crate::lowering) fn lower_pattern(
        &mut self,
        value: ValueId,
        pattern: &Pattern,
        failure: BlockId,
    ) -> Result<(), CompileError> {
        let span = pattern.span();
        let info = self
            .pattern_infos
            .get(&fpas_sema::pattern_lookup_key(pattern))
            .cloned()
            .ok_or_else(|| unsupported(span, "missing checked pattern"))?;
        let ty = self.type_table.id(&info.ty, span.line, span.column)?;
        let value = self.coerce_value_type(value, ty, span)?;
        match pattern {
            Pattern::Binding { name, .. } => self.write_named_local(name, value, span),
            Pattern::Wildcard(_) => Ok(()),
            _ if info.variant.is_some() => {
                let Some(variant) = info.variant else {
                    return Err(unsupported(span, "pattern variant"));
                };
                let saved = self.declare_hidden_local(ty, span)?;
                self.write_local(saved, value, span)?;
                let (matched, payloads) =
                    self.pattern_discriminator(value, ty, variant, &info.ty, span)?;
                self.pattern_continue(matched, failure, span)?;
                if let Pattern::Variant { arguments, .. } = pattern {
                    for (index, argument) in arguments.iter().enumerate() {
                        let source = self.emit_value(Operation::ReadLocal(saved), ty, span)?;
                        let storage = *payloads
                            .get(index)
                            .ok_or_else(|| unsupported(span, "pattern payload"))?;
                        let operation = match (self.type_kind(ty), variant) {
                            (Some(IrType::Option(_)), PatternVariant::Some) => {
                                Operation::UnwrapSome(source)
                            }
                            (Some(IrType::Result { .. }), PatternVariant::Ok) => {
                                Operation::UnwrapOk(source)
                            }
                            (Some(IrType::Result { .. }), PatternVariant::Error) => {
                                Operation::UnwrapError(source)
                            }
                            (Some(IrType::Enum(layout)), PatternVariant::Enum(index_variant)) => {
                                Operation::LoadEnumField {
                                    value: source,
                                    layout,
                                    variant: fpas_ir::VariantId::try_from_index(index_variant)
                                        .map_err(|_| unsupported(span, "pattern variant index"))?,
                                    field: FieldId::try_from_index(index)
                                        .map_err(|_| unsupported(span, "pattern payload index"))?,
                                }
                            }
                            _ => return Err(unsupported(span, "pattern payload operation")),
                        };
                        let value = self.emit_value(operation, storage, argument.span())?;
                        self.lower_pattern(value, argument, failure)?;
                    }
                }
                Ok(())
            }
            Pattern::Value { start, end, .. } => {
                let saved = self.save_value(value);
                let other = self.lower_expression_as(start, ty)?;
                let value = self.restore_value(saved, span)?;
                let matched = if let Some(end) = end {
                    let (lower, upper) = match info.ty {
                        Ty::Integer => (
                            BinaryOperation::GreaterEqualInteger,
                            BinaryOperation::LessEqualInteger,
                        ),
                        _ => (
                            BinaryOperation::GreaterEqualDynamic,
                            BinaryOperation::LessEqualDynamic,
                        ),
                    };
                    let lower = self.emit_binary(lower, value, other, types::BOOLEAN, span)?;
                    let saved_lower = self.save_value(lower);
                    let saved_value = self.save_value(value);
                    let end = self.lower_expression_as(end, ty)?;
                    let value = self.restore_value(saved_value, span)?;
                    let lower = self.restore_value(saved_lower, span)?;
                    let upper = self.emit_binary(upper, value, end, types::BOOLEAN, span)?;
                    self.emit_binary(
                        BinaryOperation::AndBoolean,
                        lower,
                        upper,
                        types::BOOLEAN,
                        span,
                    )?
                } else {
                    self.emit_binary(BinaryOperation::Equal, value, other, types::BOOLEAN, span)?
                };
                self.pattern_continue(matched, failure, span)
            }
            Pattern::Variant { .. } => Err(unsupported(span, "unresolved pattern variant")),
        }
    }

    fn pattern_continue(
        &mut self,
        matched: ValueId,
        failure: BlockId,
        span: Span,
    ) -> Result<(), CompileError> {
        let continuation = self.new_block(span)?;
        self.terminate(Terminator::Branch {
            condition: matched,
            then_target: target(continuation),
            else_target: target(failure),
        })?;
        self.switch_to(continuation);
        Ok(())
    }

    fn pattern_discriminator(
        &mut self,
        value: ValueId,
        ty: TypeId,
        variant: PatternVariant,
        semantic: &Ty,
        span: Span,
    ) -> Result<(ValueId, Vec<TypeId>), CompileError> {
        let (matched, payloads, inverse) = match (self.type_kind(ty), variant) {
            (Some(IrType::Option(payload)), PatternVariant::Some | PatternVariant::None) => (
                self.emit_value(Operation::IsOptionSome(value), types::BOOLEAN, span)?,
                vec![payload],
                variant == PatternVariant::None,
            ),
            (Some(IrType::Result { ok, error }), PatternVariant::Ok | PatternVariant::Error) => (
                self.emit_value(Operation::IsResultOk(value), types::BOOLEAN, span)?,
                vec![if variant == PatternVariant::Ok {
                    ok
                } else {
                    error
                }],
                variant == PatternVariant::Error,
            ),
            (Some(IrType::Enum(layout)), PatternVariant::Enum(index)) => {
                let Ty::Enum(enumeration) = semantic else {
                    return Err(unsupported(span, "pattern enum type"));
                };
                let variant = enumeration
                    .variants
                    .get(index)
                    .ok_or_else(|| unsupported(span, "pattern variant"))?;
                let (variant, payloads) = self
                    .enum_variant(layout, &variant.name)
                    .ok_or_else(|| unsupported(span, "pattern enum layout"))?;
                (
                    self.emit_value(
                        Operation::TestVariant {
                            value,
                            layout,
                            variant,
                        },
                        types::BOOLEAN,
                        span,
                    )?,
                    payloads,
                    false,
                )
            }
            (Some(IrType::Integer), PatternVariant::Enum(index)) => {
                let Ty::Enum(enumeration) = semantic else {
                    return Err(unsupported(span, "pattern enum type"));
                };
                let backing_value = enumeration
                    .variants
                    .get(index)
                    .and_then(|variant| variant.backing_value)
                    .ok_or_else(|| unsupported(span, "pattern enum backing value"))?;
                let ordinal = self.emit_value(
                    Operation::Const(Constant::Integer(backing_value)),
                    types::INTEGER,
                    span,
                )?;
                (
                    self.emit_binary(BinaryOperation::Equal, value, ordinal, types::BOOLEAN, span)?,
                    vec![],
                    false,
                )
            }
            _ => return Err(unsupported(span, "pattern discriminator")),
        };
        let matched = if inverse {
            self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NotBoolean,
                    operand: matched,
                },
                types::BOOLEAN,
                span,
            )?
        } else {
            matched
        };
        Ok((matched, payloads))
    }
}
