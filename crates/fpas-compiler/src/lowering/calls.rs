//! Direct and first-class call lowering.

use fpas_ir::{Operation, TypeId, ValueId};
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
        if let Some(target) = self.method_calls.get(&call_key).cloned() {
            return self.lower_method_call(designator, arguments, &target, span);
        }
        if let Some(fpas_ir::IrType::Enum(layout)) = self.type_kind(result) {
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
        let qualified = designator
            .parts
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.join("."))
            .ok_or_else(|| unsupported(designator.span, "method or qualified call"))?;
        let name = qualified.as_str();
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
