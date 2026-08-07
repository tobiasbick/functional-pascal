//! Capture-type recovery from semantically typed closure uses.

use fpas_parser::{DesignatorPart, Expr, FuncBody, PostfixOperation, Stmt};
use fpas_sema::{AnalysisMetadata, Ty};

pub(in crate::lowering) fn find_capture_type(
    body: &FuncBody,
    name: &str,
    metadata: &AnalysisMetadata,
) -> Option<Ty> {
    let FuncBody::Block { stmts, .. } = body;
    stmts
        .iter()
        .find_map(|statement| find_in_statement(statement, name, metadata))
}

fn find_in_statement(statement: &Stmt, name: &str, metadata: &AnalysisMetadata) -> Option<Ty> {
    match statement {
        Stmt::Block(statements, _)
        | Stmt::Repeat {
            body: statements, ..
        } => statements
            .iter()
            .find_map(|statement| find_in_statement(statement, name, metadata)),
        Stmt::Var(definition) | Stmt::MutableVar(definition) => {
            find_in_expression(&definition.value, name, metadata)
        }
        Stmt::Assign { target, value, .. } => find_in_designator(&target.parts, name, metadata)
            .or_else(|| find_in_expression(value, name, metadata)),
        Stmt::Return(value, _) => value
            .as_ref()
            .and_then(|value| find_in_expression(value, name, metadata)),
        Stmt::Panic(value, _)
        | Stmt::Expression { expr: value, .. }
        | Stmt::Go { expr: value, .. } => find_in_expression(value, name, metadata),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => find_in_expression(condition, name, metadata)
            .or_else(|| find_in_statement(then_branch, name, metadata))
            .or_else(|| {
                else_branch
                    .as_deref()
                    .and_then(|branch| find_in_statement(branch, name, metadata))
            }),
        Stmt::While {
            condition, body, ..
        } => find_in_expression(condition, name, metadata)
            .or_else(|| find_in_statement(body, name, metadata)),
        Stmt::For {
            start, end, body, ..
        } => find_in_expression(start, name, metadata)
            .or_else(|| find_in_expression(end, name, metadata))
            .or_else(|| find_in_statement(body, name, metadata)),
        Stmt::ForIn { iterable, body, .. } => find_in_expression(iterable, name, metadata)
            .or_else(|| find_in_statement(body, name, metadata)),
        Stmt::Call {
            designator, args, ..
        } => find_in_designator(&designator.parts, name, metadata).or_else(|| {
            args.iter()
                .find_map(|argument| find_in_expression(argument, name, metadata))
        }),
        Stmt::Case {
            expr,
            arms,
            else_body,
            ..
        } => find_in_expression(expr, name, metadata)
            .or_else(|| {
                arms.iter()
                    .find_map(|arm| find_in_statement(&arm.body, name, metadata))
            })
            .or_else(|| {
                else_body.as_ref().and_then(|body| {
                    body.iter()
                        .find_map(|statement| find_in_statement(statement, name, metadata))
                })
            }),
        Stmt::Break(_) | Stmt::Continue(_) => None,
    }
}

fn find_in_expression(expression: &Expr, name: &str, metadata: &AnalysisMetadata) -> Option<Ty> {
    match expression {
        Expr::Designator(designator) if matches!(designator.parts.as_slice(), [DesignatorPart::Ident(found, _)] if found.eq_ignore_ascii_case(name)) => {
            metadata
                .expr_types
                .get(&fpas_sema::expr_lookup_key(expression))
                .cloned()
        }
        Expr::Call {
            designator, args, ..
        } => find_in_designator(&designator.parts, name, metadata).or_else(|| {
            args.iter()
                .find_map(|argument| find_in_expression(argument, name, metadata))
        }),
        Expr::UnaryOp { operand, .. }
        | Expr::Paren(operand, _)
        | Expr::Try(operand, _)
        | Expr::Go(operand, _)
        | Expr::ResultOk(operand, _)
        | Expr::ResultError(operand, _)
        | Expr::OptionSome(operand, _) => find_in_expression(operand, name, metadata),
        Expr::BinaryOp { left, right, .. } => find_in_expression(left, name, metadata)
            .or_else(|| find_in_expression(right, name, metadata)),
        Expr::Closure(closure) => find_capture_type(&closure.body, name, metadata),
        Expr::ArrayLiteral(values, _) => values
            .iter()
            .find_map(|value| find_in_expression(value, name, metadata)),
        Expr::DictLiteral(values, _) => values.iter().find_map(|(key, value)| {
            find_in_expression(key, name, metadata)
                .or_else(|| find_in_expression(value, name, metadata))
        }),
        Expr::RecordLiteral { fields, .. } => fields
            .iter()
            .find_map(|field| find_in_expression(&field.value, name, metadata)),
        Expr::RecordUpdate { base, fields, .. } => find_in_expression(base, name, metadata)
            .or_else(|| {
                fields
                    .iter()
                    .find_map(|field| find_in_expression(&field.value, name, metadata))
            }),
        Expr::Postfix {
            base, operations, ..
        } => find_in_expression(base, name, metadata).or_else(|| {
            operations.iter().find_map(|operation| match operation {
                PostfixOperation::Index { index, .. } => find_in_expression(index, name, metadata),
                PostfixOperation::MethodCall { args, .. } => args
                    .iter()
                    .find_map(|argument| find_in_expression(argument, name, metadata)),
                PostfixOperation::Field { .. } => None,
            })
        }),
        _ => None,
    }
}

fn find_in_designator(
    parts: &[DesignatorPart],
    name: &str,
    metadata: &AnalysisMetadata,
) -> Option<Ty> {
    parts.iter().find_map(|part| match part {
        DesignatorPart::Index(index, _) => find_in_expression(index, name, metadata),
        DesignatorPart::Ident(_, _) => None,
    })
}
