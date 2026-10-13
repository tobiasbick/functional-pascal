//! Lexical declarations nested inside values and decision expressions.

use fpas_parser::{Designator, DesignatorPart, Expr, PostfixOperation};

use super::patterns::collect_case_bindings;
use crate::{DocumentSnapshot, DocumentSymbol};

/// Descends through every expression container to collect scoped declarations.
pub(super) fn collect_expression_symbols(
    snapshot: &DocumentSnapshot,
    owner: &str,
    expression: &Expr,
    output: &mut Vec<DocumentSymbol>,
) {
    match expression {
        Expr::If {
            branches,
            else_value,
            ..
        } => {
            for branch in branches {
                collect_expression_symbols(snapshot, owner, &branch.condition, output);
                collect_expression_symbols(snapshot, owner, &branch.value, output);
            }
            collect_expression_symbols(snapshot, owner, else_value, output);
        }
        Expr::Case {
            selector,
            arms,
            else_arm,
            ..
        } => {
            collect_expression_symbols(snapshot, owner, selector, output);
            for arm in arms {
                collect_case_bindings(
                    snapshot,
                    owner,
                    &arm.labels,
                    arm.span.diagnostic_span_or_synthetic(),
                    output,
                );
                if let Some(guard) = &arm.guard {
                    collect_expression_symbols(snapshot, owner, guard, output);
                }
                collect_expression_symbols(snapshot, owner, &arm.value, output);
            }
            if let Some(arm) = else_arm {
                collect_expression_symbols(snapshot, owner, &arm.value, output);
            }
        }
        Expr::Designator(designator) | Expr::VarArgument { designator, .. } => {
            collect_designator_symbols(snapshot, owner, designator, output);
        }
        Expr::Call {
            designator, args, ..
        } => {
            collect_designator_symbols(snapshot, owner, designator, output);
            for argument in args {
                collect_expression_symbols(snapshot, owner, argument, output);
            }
        }
        Expr::UnaryOp { operand: value, .. }
        | Expr::NamedArgument { value, .. }
        | Expr::Is { value, .. }
        | Expr::Paren(value, _)
        | Expr::ResultOk(value, _)
        | Expr::ResultError(value, _)
        | Expr::OptionSome(value, _)
        | Expr::Try(value, _)
        | Expr::Go(value, _) => collect_expression_symbols(snapshot, owner, value, output),
        Expr::BinaryOp { left, right, .. } => {
            collect_expression_symbols(snapshot, owner, left, output);
            collect_expression_symbols(snapshot, owner, right, output);
        }
        Expr::ArrayLiteral(values, _) => {
            for value in values {
                collect_expression_symbols(snapshot, owner, value, output);
            }
        }
        Expr::DictLiteral(entries, _) => {
            for (key, value) in entries {
                collect_expression_symbols(snapshot, owner, key, output);
                collect_expression_symbols(snapshot, owner, value, output);
            }
        }
        Expr::RecordUpdate { base, fields, .. } => {
            collect_expression_symbols(snapshot, owner, base, output);
            for field in fields {
                collect_expression_symbols(snapshot, owner, &field.value, output);
            }
        }
        Expr::Postfix {
            base, operations, ..
        } => {
            collect_expression_symbols(snapshot, owner, base, output);
            for operation in operations {
                match operation {
                    PostfixOperation::Index { index, .. } => {
                        collect_expression_symbols(snapshot, owner, index, output);
                    }
                    PostfixOperation::MethodCall { args, .. } => {
                        for argument in args {
                            collect_expression_symbols(snapshot, owner, argument, output);
                        }
                    }
                    PostfixOperation::Field { .. } => {}
                }
            }
        }
        Expr::Closure(closure) => {
            output.extend(super::routines::closure_children(snapshot, owner, closure));
        }
        Expr::Integer(..)
        | Expr::Real(..)
        | Expr::Str(..)
        | Expr::Bool(..)
        | Expr::OptionNone(_)
        | Expr::Error(_) => {}
    }
}

/// Collects declarations inside index expressions of a designator.
pub(super) fn collect_designator_symbols(
    snapshot: &DocumentSnapshot,
    owner: &str,
    designator: &Designator,
    output: &mut Vec<DocumentSymbol>,
) {
    for part in &designator.parts {
        if let DesignatorPart::Index(index, _) = part {
            collect_expression_symbols(snapshot, owner, index, output);
        }
    }
}
