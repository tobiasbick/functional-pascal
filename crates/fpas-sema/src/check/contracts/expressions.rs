//! Recursive contract checks use the identities selected by ordinary semantic analysis.

use super::Checker;
use fpas_lexer::Span;
use fpas_parser::{CaseLabel, Designator, DesignatorPart, Expr, Pattern, PostfixOperation};

impl Checker {
    pub(super) fn check_contract_expression(&mut self, expr: &Expr, clause_span: Span) {
        match expr {
            Expr::Try(..) | Expr::Go(..) | Expr::Closure(..) | Expr::VarArgument { .. } => {
                self.reject_contract_expression("propagation, tasks, anonymous routines, and reference arguments are unavailable in clauses", clause_span);
            }
            Expr::Designator(designator) => {
                let key = crate::designator_lookup_key(designator);
                if self.contracts.routine_values.contains(&key) || self.contract_bound_method(key) {
                    self.reject_contract_expression("routine values and bound methods may not be formed inside clauses; read an initialized constant or parameter instead", clause_span);
                }
                self.check_contract_designator(designator, clause_span);
            }
            Expr::Call {
                designator, args, ..
            } => {
                if !self.contract_call_allowed(expr) {
                    self.reject_contract_expression("only typed construction, conversions, and the listed native receiver operations may be called", clause_span);
                }
                self.check_contract_designator(designator, clause_span);
                for argument in args {
                    self.check_contract_expression(argument.argument_value(), clause_span);
                }
                self.queue_contract_defaults(expr, args, clause_span);
            }
            Expr::UnaryOp { operand, .. }
            | Expr::Paren(operand, _)
            | Expr::ResultOk(operand, _)
            | Expr::ResultError(operand, _)
            | Expr::OptionSome(operand, _)
            | Expr::NamedArgument { value: operand, .. } => {
                self.check_contract_expression(operand, clause_span);
            }
            Expr::BinaryOp { left, right, .. } => {
                self.check_contract_expression(left, clause_span);
                self.check_contract_expression(right, clause_span);
            }
            Expr::ArrayLiteral(elements, _) => {
                for element in elements {
                    self.check_contract_expression(element, clause_span);
                }
            }
            Expr::DictLiteral(pairs, _) => {
                for (key, value) in pairs {
                    self.check_contract_expression(key, clause_span);
                    self.check_contract_expression(value, clause_span);
                }
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.check_contract_expression(base, clause_span);
                for field in fields {
                    self.check_contract_expression(&field.value, clause_span);
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                self.check_contract_expression(base, clause_span);
                for operation in operations {
                    match operation {
                        PostfixOperation::Field { .. } => {
                            if self.contract_bound_method(Self::postfix_operation_lookup_key(
                                operation,
                            )) {
                                self.reject_contract_expression(
                                    "bound methods may not be formed inside clauses",
                                    clause_span,
                                );
                            }
                        }
                        PostfixOperation::Index { index, .. } => {
                            self.check_contract_expression(index, clause_span)
                        }
                        PostfixOperation::MethodCall { args, .. } => {
                            if !self.contract_native_allowed(Self::postfix_operation_lookup_key(
                                operation,
                            )) {
                                self.reject_contract_expression(
                                    "user methods and callable values may not be called",
                                    clause_span,
                                );
                            }
                            for argument in args {
                                self.check_contract_expression(
                                    argument.argument_value(),
                                    clause_span,
                                );
                            }
                        }
                    }
                }
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                for branch in branches {
                    self.check_contract_expression(&branch.condition, clause_span);
                    self.check_contract_expression(&branch.value, clause_span);
                }
                self.check_contract_expression(else_value, clause_span);
            }
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => {
                self.check_contract_expression(selector, clause_span);
                for arm in arms {
                    for label in &arm.labels {
                        match label {
                            CaseLabel::Pattern(pattern) => {
                                self.check_contract_pattern(pattern, clause_span)
                            }
                            CaseLabel::Value { start, end, .. } => {
                                self.check_contract_expression(start, clause_span);
                                if let Some(end) = end {
                                    self.check_contract_expression(end, clause_span);
                                }
                            }
                            CaseLabel::Binding { .. } => {}
                        }
                    }
                    if let Some(guard) = &arm.guard {
                        self.check_contract_expression(guard, clause_span);
                    }
                    self.check_contract_expression(&arm.value, clause_span);
                }
                if let Some(arm) = else_arm {
                    self.check_contract_expression(&arm.value, clause_span);
                }
            }
            Expr::Is { value, pattern, .. } => {
                self.check_contract_expression(value, clause_span);
                self.check_contract_pattern(pattern, clause_span);
            }
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_)
            | Expr::Error(_) => {}
        }
    }

    fn check_contract_designator(&mut self, designator: &Designator, span: Span) {
        if !self.contract_designator_allowed(designator) {
            self.reject_contract_expression(
                "mutable globals and mutable captures may not be read",
                span,
            );
        }
        for part in &designator.parts {
            if let DesignatorPart::Index(index, _) = part {
                self.check_contract_expression(index, span);
            }
        }
    }

    fn check_contract_pattern(&mut self, pattern: &Pattern, span: Span) {
        match pattern {
            Pattern::Value(value) => self.check_contract_expression(value, span),
            Pattern::Variant { fields, .. } => {
                for field in fields {
                    self.check_contract_pattern(&field.pattern, span);
                }
            }
            Pattern::Destructure {
                payload: Some(payload),
                ..
            } => self.check_contract_pattern(payload, span),
            _ => {}
        }
    }
}
