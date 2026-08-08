//! Direct and first-class call lowering.

use fpas_ir::{Constant, IntrinsicId, Operation, TypeId, ValueId};
use fpas_parser::{Designator, DesignatorPart, Expr};

use crate::CompileError;

use super::context::{LoweringContext, unsupported};

impl LoweringContext {
    pub(super) fn lower_call(
        &mut self,
        designator: &Designator,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
        call_key: usize,
    ) -> Result<ValueId, CompileError> {
        if let Some(name) = self.intrinsic_calls.get(&call_key).cloned() {
            return self.lower_intrinsic_call(&name, arguments, result, span);
        }
        if let Some(target) = self.method_calls.get(&call_key).cloned() {
            if crate::intrinsic_catalog::resolve(target.qualified_name(), None).is_some() {
                return self.lower_intrinsic_call(target.qualified_name(), arguments, result, span);
            }
            return self.lower_method_call(designator, arguments, &target, result, span);
        }
        let qualified = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join("."));
        let resolves_callable = qualified
            .as_deref()
            .is_some_and(|name| self.resolve_callable(name).is_some());
        if !resolves_callable && let Some(fpas_ir::IrType::Enum(layout)) = self.type_kind(result) {
            let name = designator
                .parts
                .last()
                .and_then(|part| match part {
                    DesignatorPart::Ident(name, _) => Some(name.as_str()),
                    DesignatorPart::Index(_, _) => None,
                })
                .ok_or_else(|| unsupported(designator.span, "enum constructor"))?;
            if let Some((variant, _)) = self.enum_variant(layout, name) {
                let fields = self.lower_call_arguments(arguments, span)?;
                return self.emit_value(
                    Operation::MakeEnum {
                        layout,
                        variant,
                        fields,
                    },
                    result,
                    span,
                );
            }
        }
        let qualified =
            qualified.ok_or_else(|| unsupported(designator.span, "method or qualified call"))?;
        let name = qualified.as_str();
        if name
            .strip_prefix("Application.")
            .map(|member| format!("Std.Graph.Application.{member}"))
            .filter(|canonical| crate::intrinsic_catalog::resolve(canonical, None).is_some())
            .is_some()
        {
            let canonical = format!("Std.Graph.{name}");
            return self.lower_intrinsic_call(&canonical, arguments, result, span);
        }
        if self.has_binding(name) {
            let callee = self.read_named_local(name, designator.span)?;
            let values = self.lower_call_arguments(arguments, span)?;
            self.emit_value(
                Operation::CallValue {
                    callee,
                    arguments: values,
                },
                result,
                span,
            )
        } else {
            self.lower_named_call(name, designator, arguments, span)
        }
    }

    fn lower_intrinsic_call(
        &mut self,
        name: &str,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if name.eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_PUSH) {
            return self.lower_array_push(arguments, span);
        }
        if name.eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_POP) {
            return self.lower_array_pop(arguments, result, span);
        }
        let first_type = arguments.first().and_then(|argument| {
            self.expr_types
                .get(&fpas_sema::expr_lookup_key(argument))
                .cloned()
        });
        let Some(intrinsic) = crate::intrinsic_catalog::resolve(name, first_type.as_ref()) else {
            let callable = self
                .resolve_callable(name)
                .ok_or_else(|| unsupported(span, "standard-library or source-unit call"))?;
            let values = self.lower_call_arguments(arguments, span)?;
            return self.emit_value(
                Operation::CallDirect {
                    function: callable.function,
                    arguments: values,
                },
                callable.result,
                span,
            );
        };
        if matches!(
            intrinsic,
            fpas_bytecode::Intrinsic::Console(
                fpas_bytecode::ConsoleIntrinsic::Write | fpas_bytecode::ConsoleIntrinsic::WriteLn
            )
        ) {
            return self.lower_console_write(intrinsic, arguments, span);
        }
        let mut values = self.lower_call_arguments(arguments, span)?;
        if matches!(
            intrinsic,
            fpas_bytecode::Intrinsic::Str(fpas_bytecode::StrIntrinsic::Format)
        ) {
            let argument_count = i64::try_from(arguments.len().saturating_sub(1))
                .map_err(|_| unsupported(span, "format argument count overflow"))?;
            values.push(self.emit_value(
                Operation::Const(Constant::Integer(argument_count)),
                super::types::INTEGER,
                span,
            )?);
        }
        self.emit_value(
            Operation::Intrinsic {
                intrinsic: IntrinsicId::new(u32::from(u16::from(intrinsic))),
                arguments: values,
            },
            result,
            span,
        )
    }

    fn lower_array_push(
        &mut self,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let [Expr::Designator(target), value] = arguments else {
            return Err(unsupported(span, "Std.Array.Push arguments"));
        };
        let array_ty = self.mutable_array_target_type(target)?;
        let element_ty = match self.type_kind(array_ty) {
            Some(fpas_ir::IrType::Array(element)) => element,
            _ => return Err(unsupported(target.span, "mutable array target type")),
        };
        let value = match value {
            Expr::RecordLiteral { fields, span } => {
                self.lower_record_literal_as(fields, element_ty, *span)?
            }
            _ => self.lower_expression(value)?,
        };
        let array = self.lower_designator_read(target)?;
        let appended = self.emit_value(Operation::MakeArray(vec![value]), array_ty, span)?;
        self.record_call_arguments(2, span)?;
        let updated = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Concat),
            vec![array, appended],
            array_ty,
            span,
        )?;
        self.lower_designator_write(target, updated, span)?;
        self.emit_value(Operation::Const(Constant::Unit), super::types::UNIT, span)
    }

    fn lower_array_pop(
        &mut self,
        arguments: &[Expr],
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let [Expr::Designator(target)] = arguments else {
            return Err(unsupported(span, "Std.Array.Pop argument"));
        };
        let array_ty = self.mutable_array_target_type(target)?;
        let array = self.lower_designator_read(target)?;
        self.record_call_arguments(1, span)?;
        let length = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Length),
            vec![array],
            super::types::INTEGER,
            span,
        )?;
        let one = self.emit_value(
            Operation::Const(Constant::Integer(1)),
            super::types::INTEGER,
            span,
        )?;
        let last_index = self.emit_value(
            Operation::Binary {
                operation: fpas_ir::BinaryOperation::SubtractInteger,
                left: length,
                right: one,
            },
            super::types::INTEGER,
            span,
        )?;
        let popped = self.emit_value(
            Operation::IndexGet {
                collection: array,
                index: last_index,
            },
            result,
            span,
        )?;
        let zero = self.emit_value(
            Operation::Const(Constant::Integer(0)),
            super::types::INTEGER,
            span,
        )?;
        self.record_call_arguments(3, span)?;
        let shortened = self.emit_intrinsic_value(
            fpas_bytecode::Intrinsic::Array(fpas_bytecode::ArrayIntrinsic::Slice),
            vec![array, zero, last_index],
            array_ty,
            span,
        )?;
        self.lower_designator_write(target, shortened, span)?;
        Ok(popped)
    }

    fn mutable_array_target_type(&self, target: &Designator) -> Result<TypeId, CompileError> {
        let [DesignatorPart::Ident(name, _)] = target.parts.as_slice() else {
            return Err(unsupported(target.span, "mutable array target"));
        };
        self.root_type(name)
            .ok_or_else(|| unsupported(target.span, "mutable array target type"))
    }

    fn lower_console_write(
        &mut self,
        intrinsic: fpas_bytecode::Intrinsic,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let write = fpas_bytecode::Intrinsic::Console(fpas_bytecode::ConsoleIntrinsic::Write);
        let write_ln = fpas_bytecode::Intrinsic::Console(fpas_bytecode::ConsoleIntrinsic::WriteLn);
        if arguments.is_empty() {
            if intrinsic == write {
                return self.emit_value(Operation::Const(Constant::Unit), super::types::UNIT, span);
            }
            let empty = self.emit_value(
                Operation::Const(Constant::String(String::new())),
                super::types::STRING,
                span,
            )?;
            self.record_call_arguments(1, span)?;
            return self.emit_intrinsic_value(write_ln, vec![empty], super::types::UNIT, span);
        }

        let last = arguments.len() - 1;
        let mut result = None;
        for (index, argument) in arguments.iter().enumerate() {
            let value = self.lower_expression(argument)?;
            self.record_call_arguments(1, span)?;
            let operation = if intrinsic == write_ln && index == last {
                write_ln
            } else {
                write
            };
            result = Some(self.emit_intrinsic_value(
                operation,
                vec![value],
                super::types::UNIT,
                span,
            )?);
        }
        result.ok_or_else(|| unsupported(span, "empty console write lowering"))
    }

    fn emit_intrinsic_value(
        &mut self,
        intrinsic: fpas_bytecode::Intrinsic,
        arguments: Vec<ValueId>,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        self.emit_value(
            Operation::Intrinsic {
                intrinsic: IntrinsicId::new(u32::from(u16::from(intrinsic))),
                arguments,
            },
            result,
            span,
        )
    }

    fn lower_named_call(
        &mut self,
        name: &str,
        designator: &Designator,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let callable = self
            .resolve_callable(name)
            .ok_or_else(|| unsupported(designator.span, "unresolved call"))?;
        let values = self.lower_call_arguments(arguments, span)?;
        if callable.captures.is_empty() {
            return self.emit_value(
                Operation::CallDirect {
                    function: callable.function,
                    arguments: values,
                },
                callable.result,
                span,
            );
        }
        let captures = callable
            .captures
            .iter()
            .map(|capture| self.read_capture(&capture.name, designator.span))
            .collect::<Result<Vec<_>, _>>()?;
        let callee = self.emit_value(
            Operation::MakeClosure {
                function: callable.function,
                captures,
            },
            callable.value_type,
            designator.span,
        )?;
        self.emit_value(
            Operation::CallValue {
                callee,
                arguments: values,
            },
            callable.result,
            span,
        )
    }

    fn lower_call_arguments(
        &mut self,
        arguments: &[Expr],
        span: fpas_lexer::Span,
    ) -> Result<Vec<ValueId>, CompileError> {
        let values = arguments
            .iter()
            .map(|argument| self.lower_expression(argument))
            .collect::<Result<Vec<_>, _>>()?;
        self.record_call_arguments(values.len(), span)?;
        Ok(values)
    }
}
