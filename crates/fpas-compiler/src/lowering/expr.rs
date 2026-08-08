//! Scalar expression lowering with source-order evaluation.

use fpas_ir::{BinaryOperation as IrBinary, Constant, Operation, UnaryOperation, ValueId};
use fpas_parser::{BinaryOp, DesignatorPart, Expr, UnaryOp};
use fpas_sema::Ty;

use crate::CompileError;

use super::context::{LoweringContext, unsupported};
use super::types;

impl LoweringContext {
    pub(super) fn lower_expression(&mut self, expression: &Expr) -> Result<ValueId, CompileError> {
        match expression {
            Expr::Integer(value, span) => self.emit_value(
                Operation::Const(Constant::Integer(*value)),
                types::INTEGER,
                *span,
            ),
            Expr::Real(value, span) => {
                self.emit_value(Operation::Const(Constant::Real(*value)), types::REAL, *span)
            }
            Expr::Str(value, span) => self.emit_value(
                Operation::Const(Constant::String(value.clone())),
                types::STRING,
                *span,
            ),
            Expr::Bool(value, span) => self.emit_value(
                Operation::Const(Constant::Boolean(*value)),
                types::BOOLEAN,
                *span,
            ),
            Expr::Designator(designator) => {
                let designator_key = fpas_sema::designator_lookup_key(designator);
                if self.bound_method_targets.contains_key(&designator_key) {
                    return self.lower_bound_method(designator, designator_key);
                }
                if let Some(reads) = self.property_reads.get(&designator_key).cloned() {
                    return self.lower_property_read(designator, &reads);
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
                if let Some(value) = qualified.as_ref().and_then(|name| self.enum_constant(name)) {
                    return self.emit_value(
                        Operation::Const(Constant::Integer(value)),
                        types::INTEGER,
                        designator.span,
                    );
                }
                let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
                    return self.lower_designator_read(designator);
                };
                if self.has_binding(name) {
                    self.read_named_local(name, designator.span)
                } else if self.has_global(name) {
                    self.read_global(name, designator.span)
                } else if let Some(callable) = self.resolve_callable(name) {
                    let captures = callable
                        .captures
                        .iter()
                        .map(|capture| self.read_capture(&capture.name, designator.span))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.emit_value(
                        Operation::MakeClosure {
                            function: callable.function,
                            captures,
                        },
                        callable.value_type,
                        designator.span,
                    )
                } else {
                    Err(unsupported(designator.span, "unresolved designator"))
                }
            }
            Expr::Call {
                designator,
                args,
                span,
            } => {
                let call_key = fpas_sema::expr_lookup_key(expression);
                if let Some(info) = self.event_assigned.get(&call_key).cloned() {
                    return self.lower_event_assigned(args, &info, *span);
                }
                if let Some(info) = self.event_raises.get(&call_key).cloned() {
                    return self.lower_event_raise(designator, args, &info, *span);
                }
                let result = self.expression_ir_type(expression)?;
                self.lower_call(designator, args, result, *span, call_key)
            }
            Expr::Paren(inner, _) => self.lower_expression(inner),
            Expr::UnaryOp { op, operand, span } => self.lower_unary(*op, operand, *span),
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let result_ty = self.expression_ir_type(expression)?;
                self.lower_binary(*op, left, right, result_ty, *span)
            }
            Expr::Closure(_) => {
                let target = self
                    .closure_target(expression)
                    .ok_or_else(|| unsupported(expression.span(), "unregistered closure"))?;
                let captures = target
                    .captures
                    .iter()
                    .map(|capture| self.read_capture(&capture.name, expression.span()))
                    .collect::<Result<Vec<_>, _>>()?;
                self.emit_value(
                    Operation::MakeClosure {
                        function: target.function,
                        captures,
                    },
                    target.value_type,
                    expression.span(),
                )
            }
            Expr::ArrayLiteral(values, _) => self.lower_array_literal(values, expression),
            Expr::DictLiteral(values, _) => self.lower_dictionary_literal(values, expression),
            Expr::RecordLiteral { fields, .. } => self.lower_record_literal(fields, expression),
            Expr::RecordUpdate { base, fields, .. } => {
                self.lower_record_update(base, fields, expression)
            }
            Expr::ResultOk(value, _) => self.lower_wrapper(Some(value), expression, 0),
            Expr::ResultError(value, _) => self.lower_wrapper(Some(value), expression, 1),
            Expr::OptionSome(value, _) => self.lower_wrapper(Some(value), expression, 2),
            Expr::OptionNone(_) => self.lower_wrapper(None, expression, 3),
            Expr::Try(value, _) => self.lower_try(value, expression),
            Expr::Postfix {
                base,
                operations,
                span,
            } => self.lower_postfix(base, operations, *span),
            _ => Err(unsupported(expression.span(), "expression")),
        }
    }

    fn lower_unary(
        &mut self,
        operation: UnaryOp,
        operand: &Expr,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let operand_ty = self.expression_type(operand)?;
        let value = self.lower_expression(operand)?;
        let result_ty = self.expression_ir_type(operand)?;
        match (operation, operand_ty) {
            (UnaryOp::Negate, Ty::Integer) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NegateInteger,
                    operand: value,
                },
                types::INTEGER,
                span,
            ),
            (UnaryOp::Negate, Ty::Real) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NegateReal,
                    operand: value,
                },
                types::REAL,
                span,
            ),
            (UnaryOp::Negate, Ty::GenericParam(..)) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NegateDynamic,
                    operand: value,
                },
                types::DYNAMIC,
                span,
            ),
            (UnaryOp::Not, Ty::Boolean) => self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::NotBoolean,
                    operand: value,
                },
                types::BOOLEAN,
                span,
            ),
            (UnaryOp::Not, Ty::Integer) => {
                let mask = self.emit_value(
                    Operation::Const(Constant::Integer(-1)),
                    types::INTEGER,
                    span,
                )?;
                self.emit_binary(IrBinary::BitXorInteger, value, mask, result_ty, span)
            }
            _ => Err(unsupported(span, "unary operand type")),
        }
    }

    fn lower_binary(
        &mut self,
        operation: BinaryOp,
        left: &Expr,
        right: &Expr,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let left_ty = self.expression_type(left)?;
        let right_ty = self.expression_type(right)?;
        match operation {
            BinaryOp::Add => {
                if matches!((&left_ty, &right_ty), (Ty::String, Ty::String)) {
                    return self.lower_direct_binary(
                        IrBinary::ConcatString,
                        left,
                        right,
                        types::STRING,
                        span,
                    );
                }
                self.lower_numeric_binary(
                    IrBinary::AddInteger,
                    IrBinary::AddReal,
                    IrBinary::AddDynamic,
                    left,
                    right,
                    &left_ty,
                    &right_ty,
                    result_ty,
                    span,
                )
            }
            BinaryOp::Sub => self.lower_numeric_binary(
                IrBinary::SubtractInteger,
                IrBinary::SubtractReal,
                IrBinary::SubtractDynamic,
                left,
                right,
                &left_ty,
                &right_ty,
                result_ty,
                span,
            ),
            BinaryOp::Mul => self.lower_numeric_binary(
                IrBinary::MultiplyInteger,
                IrBinary::MultiplyReal,
                IrBinary::MultiplyDynamic,
                left,
                right,
                &left_ty,
                &right_ty,
                result_ty,
                span,
            ),
            BinaryOp::RealDiv => self.lower_numeric_binary(
                IrBinary::DivideReal,
                IrBinary::DivideReal,
                IrBinary::DivideDynamic,
                left,
                right,
                &left_ty,
                &right_ty,
                result_ty,
                span,
            ),
            BinaryOp::IntDiv => {
                self.lower_direct_binary(IrBinary::DivideInteger, left, right, result_ty, span)
            }
            BinaryOp::Mod => {
                self.lower_direct_binary(IrBinary::RemainderInteger, left, right, result_ty, span)
            }
            BinaryOp::Shl | BinaryOp::Shr => self.lower_direct_binary(
                if operation == BinaryOp::Shl {
                    IrBinary::ShiftLeftInteger
                } else {
                    IrBinary::ShiftRightInteger
                },
                left,
                right,
                result_ty,
                span,
            ),
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                let ir = boolean_or_bitwise_operation(operation, &left_ty)
                    .ok_or_else(|| unsupported(span, "boolean or bitwise operation"))?;
                self.lower_direct_binary(ir, left, right, result_ty, span)
            }
            BinaryOp::Eq | BinaryOp::NotEq => self.lower_numeric_comparison(
                if operation == BinaryOp::Eq {
                    IrBinary::Equal
                } else {
                    IrBinary::NotEqual
                },
                left,
                right,
                &left_ty,
                &right_ty,
                span,
            ),
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                let (integer, real, dynamic) = ordering_operations(operation)
                    .ok_or_else(|| unsupported(span, "ordering operation"))?;
                self.lower_numeric_binary(
                    integer,
                    real,
                    dynamic,
                    left,
                    right,
                    &left_ty,
                    &right_ty,
                    types::BOOLEAN,
                    span,
                )
            }
            BinaryOp::In => {
                let value = self.lower_expression(left)?;
                let collection = self.lower_expression(right)?;
                self.emit_value(
                    Operation::Contains { value, collection },
                    types::BOOLEAN,
                    span,
                )
            }
        }
    }

    fn lower_numeric_comparison(
        &mut self,
        operation: IrBinary,
        left: &Expr,
        right: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if matches!(
            (left_ty, right_ty),
            (Ty::Integer, Ty::Real) | (Ty::Real, Ty::Integer)
        ) {
            let (left_value, right_value) =
                self.lower_numeric_operands(left, right, left_ty, right_ty, span)?;
            return self.emit_binary(operation, left_value, right_value, types::BOOLEAN, span);
        }
        self.lower_direct_binary(operation, left, right, types::BOOLEAN, span)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "numeric selection keeps all typed choices explicit"
    )]
    fn lower_numeric_binary(
        &mut self,
        integer: IrBinary,
        real: IrBinary,
        dynamic: IrBinary,
        left: &Expr,
        right: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if matches!(left_ty, Ty::GenericParam(..)) || matches!(right_ty, Ty::GenericParam(..)) {
            return self.lower_direct_binary(dynamic, left, right, result_ty, span);
        }
        if matches!(left_ty, Ty::Real)
            || matches!(right_ty, Ty::Real)
            || integer == IrBinary::DivideReal
        {
            let (left_value, right_value) =
                self.lower_numeric_operands(left, right, left_ty, right_ty, span)?;
            return self.emit_binary(real, left_value, right_value, result_ty, span);
        }
        self.lower_direct_binary(integer, left, right, result_ty, span)
    }

    fn lower_numeric_operands(
        &mut self,
        left: &Expr,
        right: &Expr,
        left_ty: &Ty,
        right_ty: &Ty,
        span: fpas_lexer::Span,
    ) -> Result<(ValueId, ValueId), CompileError> {
        let left_value = self.lower_expression(left)?;
        let left_value = self.convert_integer_to_real(left_value, left_ty, span)?;
        let right_value = self.lower_expression(right)?;
        let right_value = self.convert_integer_to_real(right_value, right_ty, span)?;
        Ok((left_value, right_value))
    }

    fn convert_integer_to_real(
        &mut self,
        value: ValueId,
        ty: &Ty,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        if matches!(ty, Ty::Integer) {
            self.emit_value(
                Operation::Unary {
                    operation: UnaryOperation::IntegerToReal,
                    operand: value,
                },
                types::REAL,
                span,
            )
        } else {
            Ok(value)
        }
    }

    fn lower_direct_binary(
        &mut self,
        operation: IrBinary,
        left: &Expr,
        right: &Expr,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        let left = self.lower_expression(left)?;
        let right = self.lower_expression(right)?;
        self.emit_binary(operation, left, right, result_ty, span)
    }

    pub(super) fn emit_binary(
        &mut self,
        operation: IrBinary,
        left: ValueId,
        right: ValueId,
        result_ty: fpas_ir::TypeId,
        span: fpas_lexer::Span,
    ) -> Result<ValueId, CompileError> {
        self.emit_value(
            Operation::Binary {
                operation,
                left,
                right,
            },
            result_ty,
            span,
        )
    }
}

fn boolean_or_bitwise_operation(operation: BinaryOp, left_ty: &Ty) -> Option<IrBinary> {
    match (operation, left_ty) {
        (BinaryOp::And, Ty::Boolean) => Some(IrBinary::AndBoolean),
        (BinaryOp::Or, Ty::Boolean) => Some(IrBinary::OrBoolean),
        (BinaryOp::Xor, Ty::Boolean) => Some(IrBinary::NotEqual),
        (BinaryOp::And, _) => Some(IrBinary::BitAndInteger),
        (BinaryOp::Or, _) => Some(IrBinary::BitOrInteger),
        (BinaryOp::Xor, _) => Some(IrBinary::BitXorInteger),
        _ => None,
    }
}

fn ordering_operations(operation: BinaryOp) -> Option<(IrBinary, IrBinary, IrBinary)> {
    match operation {
        BinaryOp::Lt => Some((
            IrBinary::LessThanInteger,
            IrBinary::LessThanReal,
            IrBinary::LessThanDynamic,
        )),
        BinaryOp::Gt => Some((
            IrBinary::GreaterThanInteger,
            IrBinary::GreaterThanReal,
            IrBinary::GreaterThanDynamic,
        )),
        BinaryOp::LtEq => Some((
            IrBinary::LessEqualInteger,
            IrBinary::LessEqualReal,
            IrBinary::LessEqualDynamic,
        )),
        BinaryOp::GtEq => Some((
            IrBinary::GreaterEqualInteger,
            IrBinary::GreaterEqualReal,
            IrBinary::GreaterEqualDynamic,
        )),
        _ => None,
    }
}
