//! Scalar case lowering with ordered labels, ranges, guards, and bindings.

use fpas_ir::{BinaryOperation, Constant, Operation, Terminator};
use fpas_parser::{CaseArm, CaseLabel, DesignatorPart, DestructureVariant, Expr, Stmt};
use fpas_sema::Ty;

use crate::CompileError;

use super::context::{LoweringContext, target, unsupported};
use super::types;

type PatternBinding = (String, fpas_ir::TypeId, Operation);
type VariantPattern = (fpas_ir::ValueId, Vec<PatternBinding>);

impl LoweringContext {
    pub(super) fn lower_case(
        &mut self,
        expression: &Expr,
        arms: &[CaseArm],
        else_body: Option<&[Stmt]>,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let case_ty = self.expression_type(expression)?;
        let case_ir_ty = self.expression_ir_type(expression)?;
        let case_value = self.lower_expression(expression)?;
        if matches!(case_ty, Ty::Result(..) | Ty::Option(..))
            || matches!(&case_ty, Ty::Enum(enumeration) if enumeration.has_data())
        {
            return self.lower_variant_case(case_value, case_ir_ty, arms, else_body, span);
        }
        self.begin_scope();
        let case_local = self.declare_hidden_local(case_ir_ty, span)?;
        self.write_local(case_local, case_value, span)?;
        let merge_block = self.new_block(span)?;
        let first_test = self.new_block(span)?;
        self.jump(first_test)?;
        self.switch_to(first_test);
        let mut has_merge_predecessor = false;

        for arm in arms {
            for label in &arm.labels {
                let next_test = self.new_block(arm.span)?;
                let body_block = self.new_block(arm.span)?;
                let is_binding = self.is_scalar_binding(label);
                let matched =
                    self.lower_case_match(label, case_local, &case_ty, is_binding, span)?;
                self.terminate(Terminator::Branch {
                    condition: matched,
                    then_target: target(body_block),
                    else_target: target(next_test),
                })?;

                self.switch_to(body_block);
                if is_binding {
                    self.begin_scope();
                    let name =
                        binding_name(label).ok_or_else(|| unsupported(arm.span, "case binding"))?;
                    let value =
                        self.emit_value(Operation::ReadLocal(case_local), case_ir_ty, span)?;
                    let local = self.declare_local(name, case_ir_ty, false, arm.span)?;
                    self.write_local(local, value, arm.span)?;
                }
                if let Some(guard) = &arm.guard {
                    let guard_value = self.lower_expression(guard)?;
                    let guarded_body = self.new_block(arm.span)?;
                    self.terminate(Terminator::Branch {
                        condition: guard_value,
                        then_target: target(guarded_body),
                        else_target: target(next_test),
                    })?;
                    self.switch_to(guarded_body);
                }
                self.lower_statement(&arm.body)?;
                if !self.is_terminated() {
                    self.jump(merge_block)?;
                    has_merge_predecessor = true;
                }
                if is_binding {
                    self.end_scope();
                }
                self.switch_to(next_test);
            }
        }

        if let Some(else_body) = else_body {
            self.lower_statements(else_body)?;
        }
        if !self.is_terminated() {
            self.jump(merge_block)?;
            has_merge_predecessor = true;
        }
        self.end_scope();
        if has_merge_predecessor {
            self.switch_to(merge_block);
        }
        Ok(())
    }

    fn lower_case_match(
        &mut self,
        label: &CaseLabel,
        case_local: fpas_ir::LocalId,
        case_ty: &Ty,
        binding: bool,
        span: fpas_lexer::Span,
    ) -> Result<fpas_ir::ValueId, CompileError> {
        if binding {
            return self.emit_value(
                Operation::Const(Constant::Boolean(true)),
                types::BOOLEAN,
                span,
            );
        }
        let CaseLabel::Value { start, end, .. } = label else {
            return Err(unsupported(span, "destructuring case label"));
        };
        let left = self.emit_value(
            Operation::ReadLocal(case_local),
            types::lower(case_ty, span.line, span.column)?,
            span,
        )?;
        let start_value = self.lower_expression(start)?;
        if let Some(end) = end {
            let ge = self.case_ordering(case_ty, false);
            let lower = self.emit_binary(ge, left, start_value, types::BOOLEAN, span)?;
            let upper_left = self.emit_value(
                Operation::ReadLocal(case_local),
                types::lower(case_ty, span.line, span.column)?,
                span,
            )?;
            let end_value = self.lower_expression(end)?;
            let le = self.case_ordering(case_ty, true);
            let upper = self.emit_binary(le, upper_left, end_value, types::BOOLEAN, span)?;
            self.emit_binary(
                BinaryOperation::AndBoolean,
                lower,
                upper,
                types::BOOLEAN,
                span,
            )
        } else {
            self.emit_binary(
                BinaryOperation::Equal,
                left,
                start_value,
                types::BOOLEAN,
                span,
            )
        }
    }

    fn case_ordering(&self, ty: &Ty, upper: bool) -> BinaryOperation {
        match (ty, upper) {
            (Ty::Integer, false) => BinaryOperation::GreaterEqualInteger,
            (Ty::Integer, true) => BinaryOperation::LessEqualInteger,
            (Ty::Real, false) => BinaryOperation::GreaterEqualReal,
            (Ty::Real, true) => BinaryOperation::LessEqualReal,
            (_, false) => BinaryOperation::GreaterEqualDynamic,
            (_, true) => BinaryOperation::LessEqualDynamic,
        }
    }

    fn is_scalar_binding(&self, label: &CaseLabel) -> bool {
        let CaseLabel::Value { start, .. } = label else {
            return false;
        };
        self.scalar_case_bindings
            .contains(&fpas_sema::expr_lookup_key(start))
    }

    fn lower_variant_case(
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
        }
        if !self.is_terminated() {
            self.jump(merge)?;
            has_merge = true;
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

fn binding_name(label: &CaseLabel) -> Option<&str> {
    let CaseLabel::Value {
        start: Expr::Designator(designator),
        ..
    } = label
    else {
        return None;
    };
    let [fpas_parser::DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
        return None;
    };
    Some(name)
}
