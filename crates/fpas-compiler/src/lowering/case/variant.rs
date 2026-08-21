//! Result, Option, and data-enum variant case lowering.

use fpas_ir::{Constant, Operation, Terminator};
use fpas_parser::{CaseArm, CaseLabel, DesignatorPart, DestructureVariant, Expr, Stmt};

use crate::CompileError;

use super::super::context::{LoweringContext, target, unsupported};
use super::super::types;

type PatternBinding = (String, fpas_ir::TypeId, Operation);
type VariantPattern = (fpas_ir::ValueId, Vec<PatternBinding>);

impl LoweringContext {
    pub(in crate::lowering) fn lower_variant_case(
        &mut self,
        case_value: fpas_ir::ValueId,
        case_ty: fpas_ir::TypeId,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let case_local = self.declare_hidden_local(case_ty, span)?;
        self.write_local(case_local, case_value, span)?;
        let merge = self.new_block(span)?;
        let first_test = self.new_block(span)?;
        self.jump(first_test)?;
        self.switch_to(first_test);
        let mut has_merge = false;
        for arm in arms {
            for label in &arm.labels {
                let next = self.new_block(arm.span)?;
                let body = self.new_block(arm.span)?;
                let value = self.emit_value(Operation::ReadLocal(case_local), case_ty, arm.span)?;
                let (matched, bindings) =
                    self.lower_variant_pattern(value, case_ty, label, arm.span)?;
                self.terminate(Terminator::Branch {
                    condition: matched,
                    then_target: target(body),
                    else_target: target(next),
                })?;
                self.switch_to(body);
                self.begin_scope();
                for (name, ty, operation) in bindings {
                    let source =
                        self.emit_value(Operation::ReadLocal(case_local), case_ty, arm.span)?;
                    let operation = binding_operation(operation, source)?;
                    let value = self.emit_value(operation, ty, arm.span)?;
                    let local = self.declare_local(&name, ty, false, arm.span)?;
                    self.write_local(local, value, arm.span)?;
                }
                if let Some(guard) = &arm.guard {
                    let condition = self.lower_expression(guard)?;
                    let guarded = self.new_block(arm.span)?;
                    self.terminate(Terminator::Branch {
                        condition,
                        then_target: target(guarded),
                        else_target: target(next),
                    })?;
                    self.switch_to(guarded);
                }
                self.lower_statement(&arm.body)?;
                if !self.is_terminated() {
                    self.jump(merge)?;
                    has_merge = true;
                }
                self.end_scope();
                self.switch_to(next);
            }
        }
        if let Some(statements) = else_body {
            self.lower_statements(statements)?;
            if !self.is_terminated() {
                self.jump(merge)?;
                has_merge = true;
            }
        } else if !self.is_terminated() {
            let message = self.emit_value(
                Operation::Const(Constant::String(
                    "exhaustive case reached no variant".to_string(),
                )),
                types::STRING,
                span,
            )?;
            self.terminate(Terminator::Panic(message))?;
        }
        if has_merge {
            self.switch_to(merge);
        }
        Ok(())
    }

    fn lower_variant_pattern(
        &mut self,
        value: fpas_ir::ValueId,
        ty: fpas_ir::TypeId,
        label: &CaseLabel,
        span: fpas_lexer::Span,
    ) -> Result<VariantPattern, CompileError> {
        match (self.type_kind(ty), label) {
            (
                Some(fpas_ir::IrType::Result { ok, error }),
                CaseLabel::Destructure {
                    variant, binding, ..
                },
            ) => {
                let is_ok = self.emit_value(Operation::IsResultOk(value), types::BOOLEAN, span)?;
                let (matched, payload, unwrap) = match variant {
                    DestructureVariant::Ok => (is_ok, ok, Operation::UnwrapOk(value)),
                    DestructureVariant::Error => {
                        let inverse = self.emit_value(
                            Operation::Unary {
                                operation: fpas_ir::UnaryOperation::NotBoolean,
                                operand: is_ok,
                            },
                            types::BOOLEAN,
                            span,
                        )?;
                        (inverse, error, Operation::UnwrapError(value))
                    }
                    _ => return Err(unsupported(span, "Result pattern")),
                };
                let bindings = binding
                    .iter()
                    .map(|name| (name.clone(), payload, unwrap.clone()))
                    .collect();
                Ok((matched, bindings))
            }
            (
                Some(fpas_ir::IrType::Option(payload)),
                CaseLabel::Destructure {
                    variant, binding, ..
                },
            ) => {
                let is_some =
                    self.emit_value(Operation::IsOptionSome(value), types::BOOLEAN, span)?;
                let matched = match variant {
                    DestructureVariant::Some => is_some,
                    DestructureVariant::None => self.emit_value(
                        Operation::Unary {
                            operation: fpas_ir::UnaryOperation::NotBoolean,
                            operand: is_some,
                        },
                        types::BOOLEAN,
                        span,
                    )?,
                    _ => return Err(unsupported(span, "Option pattern")),
                };
                let bindings = if *variant == DestructureVariant::Some {
                    binding
                        .iter()
                        .map(|name| (name.clone(), payload, Operation::UnwrapSome(value)))
                        .collect()
                } else {
                    Vec::new()
                };
                Ok((matched, bindings))
            }
            (
                Some(fpas_ir::IrType::Enum(layout)),
                CaseLabel::Value {
                    start, end: None, ..
                },
            ) => {
                let (name, args) = match start {
                    Expr::Call {
                        designator, args, ..
                    } => (variant_name(designator)?, args.as_slice()),
                    Expr::Designator(designator) => (variant_name(designator)?, &[][..]),
                    _ => return Err(unsupported(span, "enum pattern")),
                };
                let (variant, fields) = self
                    .enum_variant(layout, name)
                    .ok_or_else(|| unsupported(span, "enum pattern variant"))?;
                let matched = self.emit_value(
                    Operation::TestVariant {
                        value,
                        layout,
                        variant,
                    },
                    types::BOOLEAN,
                    span,
                )?;
                let bindings = args
                    .iter()
                    .enumerate()
                    .filter_map(|(index, arg)| match arg {
                        Expr::Designator(designator) if designator.parts.len() == 1 => {
                            match &designator.parts[0] {
                                DesignatorPart::Ident(name, _) if name != "_" => {
                                    Some((index, name.clone()))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .map(|(index, name)| {
                        let field = fpas_ir::FieldId::try_from_index(index)
                            .map_err(|_| unsupported(span, "enum pattern field"))?;
                        let field_ty = fields
                            .get(index)
                            .copied()
                            .ok_or_else(|| unsupported(span, "enum pattern field"))?;
                        Ok((
                            name,
                            field_ty,
                            Operation::LoadEnumField {
                                value,
                                layout,
                                variant,
                                field,
                            },
                        ))
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?;
                Ok((matched, bindings))
            }
            _ => Err(unsupported(span, "variant case pattern")),
        }
    }
}

fn variant_name(designator: &fpas_parser::Designator) -> Result<&str, CompileError> {
    designator
        .parts
        .last()
        .and_then(|part| match part {
            DesignatorPart::Ident(name, _) => Some(name.as_str()),
            _ => None,
        })
        .ok_or_else(|| unsupported(designator.span, "enum pattern name"))
}

fn binding_operation(
    operation: Operation,
    source: fpas_ir::ValueId,
) -> Result<Operation, CompileError> {
    match operation {
        Operation::UnwrapOk(_) => Ok(Operation::UnwrapOk(source)),
        Operation::UnwrapError(_) => Ok(Operation::UnwrapError(source)),
        Operation::UnwrapSome(_) => Ok(Operation::UnwrapSome(source)),
        Operation::LoadEnumField {
            layout,
            variant,
            field,
            ..
        } => Ok(Operation::LoadEnumField {
            value: source,
            layout,
            variant,
            field,
        }),
        _ => Err(unsupported(
            fpas_lexer::Span {
                offset: 0,
                length: 0,
                line: 1,
                column: 1,
                source_id: 0,
            },
            "variant binding operation",
        )),
    }
}
