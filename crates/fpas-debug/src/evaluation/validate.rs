//! Exhaustive parser-AST validation and safe-IR lowering.

use fpas_parser::{BinaryOp, Designator, DesignatorPart, Expr, PostfixOperation, UnaryOp};
use fpas_vm::{DebugBinaryOperation, DebugEvaluationLimits, DebugExpression, DebugUnaryOperation};

use super::parse::EvaluationParseError;

pub(super) fn validate_expression(
    expression: &Expr,
    limits: DebugEvaluationLimits,
) -> Result<DebugExpression, EvaluationParseError> {
    let mut budget = ValidationBudget {
        operations: 0,
        traversals: 0,
    };
    lower(expression, 0, limits, &mut budget)
}

fn lower(
    expression: &Expr,
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut ValidationBudget,
) -> Result<DebugExpression, EvaluationParseError> {
    check_budget(expression, depth, limits, budget)?;
    match expression {
        Expr::Integer(value, _) => Ok(DebugExpression::Integer(*value)),
        Expr::Real(value, _) => Ok(DebugExpression::Real(*value)),
        Expr::Str(value, _) => Ok(DebugExpression::String(value.clone())),
        Expr::Bool(value, _) => Ok(DebugExpression::Boolean(*value)),
        Expr::Designator(designator) => lower_designator(designator, depth, limits, budget),
        Expr::UnaryOp { op, operand, .. } => Ok(DebugExpression::Unary {
            operation: match op {
                UnaryOp::Not => DebugUnaryOperation::Not,
                UnaryOp::Negate => DebugUnaryOperation::Negate,
            },
            operand: Box::new(lower(operand, depth + 1, limits, budget)?),
        }),
        Expr::BinaryOp {
            op, left, right, ..
        } => Ok(DebugExpression::Binary {
            operation: lower_binary(*op),
            left: Box::new(lower(left, depth + 1, limits, budget)?),
            right: Box::new(lower(right, depth + 1, limits, budget)?),
        }),
        Expr::Paren(inner, _) => lower(inner, depth + 1, limits, budget),
        Expr::Postfix {
            base, operations, ..
        } => {
            let mut lowered = lower(base, depth + 1, limits, budget)?;
            for operation in operations {
                count_traversal(expression, limits, budget)?;
                lowered = match operation {
                    PostfixOperation::Field { name, .. } => DebugExpression::Field {
                        base: Box::new(lowered),
                        name: name.clone(),
                    },
                    PostfixOperation::Index { index, .. } => DebugExpression::Index {
                        base: Box::new(lowered),
                        index: Box::new(lower(index, depth + 1, limits, budget)?),
                    },
                    PostfixOperation::MethodCall { .. } => {
                        return Err(unsupported(
                            expression,
                            "method calls",
                            "Read a stored field, for example `Point.X`.",
                        ));
                    }
                };
            }
            Ok(lowered)
        }
        Expr::Call { .. } => Err(unsupported(
            expression,
            "function and procedure calls",
            "Use visible values and operators only, for example `Counter + 1`.",
        )),
        Expr::ArrayLiteral(_, _) => Err(unsupported(
            expression,
            "array construction",
            "Read an existing array element, for example `Items[0]`.",
        )),
        Expr::DictLiteral(_, _) => Err(unsupported(
            expression,
            "dictionary construction",
            "Read an existing dictionary entry, for example `Items['key']`.",
        )),
        Expr::RecordLiteral { .. } => Err(unsupported(
            expression,
            "record construction",
            "Read an existing stored field, for example `Point.X`.",
        )),
        Expr::ResultOk(_, _)
        | Expr::ResultError(_, _)
        | Expr::OptionSome(_, _)
        | Expr::OptionNone(_)
        | Expr::Nil(_) => Err(unsupported(
            expression,
            "wrapper construction",
            "Use a visible scalar or aggregate value.",
        )),
        Expr::Try(_, _) => Err(unsupported(
            expression,
            "`try` evaluation",
            "Inspect the Result or Option value without unwrapping it.",
        )),
        Expr::Go(_, _) => Err(unsupported(
            expression,
            "task spawning with `go`",
            "Use a read-only expression without task operations.",
        )),
        Expr::RecordUpdate { .. } => Err(unsupported(
            expression,
            "record updates",
            "Read an existing stored field without creating a new record.",
        )),
        Expr::Closure(_) => Err(unsupported(
            expression,
            "closure construction",
            "Use a visible scalar or aggregate value.",
        )),
        Expr::Error(_) => Err(unsupported(
            expression,
            "recovered parser nodes",
            "Fix the expression syntax before evaluating it.",
        )),
    }
}

fn lower_designator(
    designator: &Designator,
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut ValidationBudget,
) -> Result<DebugExpression, EvaluationParseError> {
    let Some(DesignatorPart::Ident(name, _)) = designator.parts.first() else {
        return Err(unsupported_designator(designator));
    };
    let mut lowered = DebugExpression::Name(name.clone());
    for part in &designator.parts[1..] {
        count_traversal_span(
            designator.span.offset,
            designator.span.length,
            limits,
            budget,
        )?;
        lowered = match part {
            DesignatorPart::Ident(name, _) => DebugExpression::Field {
                base: Box::new(lowered),
                name: name.clone(),
            },
            DesignatorPart::Index(index, _) => DebugExpression::Index {
                base: Box::new(lowered),
                index: Box::new(lower(index, depth + 1, limits, budget)?),
            },
        };
    }
    Ok(lowered)
}

fn lower_binary(operation: BinaryOp) -> DebugBinaryOperation {
    match operation {
        BinaryOp::Mul => DebugBinaryOperation::Multiply,
        BinaryOp::RealDiv => DebugBinaryOperation::RealDivide,
        BinaryOp::IntDiv => DebugBinaryOperation::IntegerDivide,
        BinaryOp::Mod => DebugBinaryOperation::Modulo,
        BinaryOp::And => DebugBinaryOperation::And,
        BinaryOp::Shl => DebugBinaryOperation::ShiftLeft,
        BinaryOp::Shr => DebugBinaryOperation::ShiftRight,
        BinaryOp::Add => DebugBinaryOperation::Add,
        BinaryOp::Sub => DebugBinaryOperation::Subtract,
        BinaryOp::Or => DebugBinaryOperation::Or,
        BinaryOp::Xor => DebugBinaryOperation::Xor,
        BinaryOp::Eq => DebugBinaryOperation::Equal,
        BinaryOp::NotEq => DebugBinaryOperation::NotEqual,
        BinaryOp::Lt => DebugBinaryOperation::Less,
        BinaryOp::Gt => DebugBinaryOperation::Greater,
        BinaryOp::LtEq => DebugBinaryOperation::LessEqual,
        BinaryOp::GtEq => DebugBinaryOperation::GreaterEqual,
        BinaryOp::In => DebugBinaryOperation::In,
    }
}

fn check_budget(
    expression: &Expr,
    depth: usize,
    limits: DebugEvaluationLimits,
    budget: &mut ValidationBudget,
) -> Result<(), EvaluationParseError> {
    if depth > limits.max_depth {
        return Err(limit(
            expression,
            format!("debug expression depth exceeds limit {}", limits.max_depth),
            "Use a shallower watch expression.",
        ));
    }
    budget.operations = budget.operations.saturating_add(1);
    if budget.operations > limits.max_operations {
        return Err(limit(
            expression,
            format!(
                "debug expression operation count exceeds limit {}",
                limits.max_operations
            ),
            "Use a smaller watch expression.",
        ));
    }
    Ok(())
}

fn count_traversal(
    expression: &Expr,
    limits: DebugEvaluationLimits,
    budget: &mut ValidationBudget,
) -> Result<(), EvaluationParseError> {
    let span = expression.span();
    count_traversal_span(span.offset, span.length, limits, budget)
}

fn count_traversal_span(
    offset: usize,
    length: usize,
    limits: DebugEvaluationLimits,
    budget: &mut ValidationBudget,
) -> Result<(), EvaluationParseError> {
    budget.traversals = budget.traversals.saturating_add(1);
    if budget.traversals > limits.max_traversals {
        return Err(EvaluationParseError {
            code: "evaluation_limit",
            message: format!(
                "debug expression aggregate traversal count exceeds limit {}",
                limits.max_traversals
            ),
            hint: "Use fewer field and index operations.".to_string(),
            offset,
            length,
        });
    }
    Ok(())
}

fn unsupported(expression: &Expr, construct: &str, hint: &str) -> EvaluationParseError {
    let span = expression.span();
    EvaluationParseError {
        code: "unsupported_expression",
        message: format!("debugger evaluation does not support {construct}"),
        hint: hint.to_string(),
        offset: span.offset,
        length: span.length,
    }
}

fn unsupported_designator(designator: &Designator) -> EvaluationParseError {
    EvaluationParseError {
        code: "unsupported_expression",
        message: "debugger evaluation requires a named designator root".to_string(),
        hint: "Start with a visible binding name.".to_string(),
        offset: designator.span.offset,
        length: designator.span.length,
    }
}

fn limit(expression: &Expr, message: String, hint: &str) -> EvaluationParseError {
    let span = expression.span();
    EvaluationParseError {
        code: "evaluation_limit",
        message,
        hint: hint.to_string(),
        offset: span.offset,
        length: span.length,
    }
}

struct ValidationBudget {
    operations: usize,
    traversals: usize,
}
