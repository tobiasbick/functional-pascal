//! Validates known logical operand types without running skipped debugger expressions.
//!
//! Operator semantics: `docs/pascal/language/basics/operators.md`.

use fpas_parser::Expr;
use fpas_vm::{DebugBinaryOperation, DebugExpression, DebugUnaryOperation};

use super::parse::EvaluationParseError;

/// Rejects known non-Boolean operands before evaluating a logical watch expression.
pub(super) fn validate(
    expression: &Expr,
    lowered: &DebugExpression,
) -> Result<(), EvaluationParseError> {
    let (operands, replacement): (Vec<&DebugExpression>, _) = match lowered {
        DebugExpression::Unary {
            operation: DebugUnaryOperation::Not,
            operand,
        } => (vec![operand], "BitNot"),
        DebugExpression::Binary {
            operation,
            left,
            right,
        } => {
            let replacement = match operation {
                DebugBinaryOperation::And => "BitAnd",
                DebugBinaryOperation::Or => "BitOr",
                DebugBinaryOperation::Xor => "BitXor",
                _ => return Ok(()),
            };
            (vec![left, right], replacement)
        }
        _ => return Ok(()),
    };
    if !operands
        .iter()
        .any(|operand| matches!(kind(operand), Kind::Integer | Kind::Other))
    {
        return Ok(());
    }
    let hint = if operands
        .iter()
        .all(|operand| kind(operand) == Kind::Integer)
    {
        format!("Use `Std.Bits.{replacement}` and import `uses Std.Bits;` for integer bits.")
    } else {
        "Use Boolean operands; compare numeric values explicitly, such as `Count > 0`.".to_string()
    };
    let span = expression.span();
    Err(EvaluationParseError {
        code: "evaluation_type",
        message: "Logical operators require Boolean operands".to_string(),
        hint,
        offset: span.offset,
        length: span.length,
    })
}

#[derive(PartialEq, Eq)]
enum Kind {
    Boolean,
    Integer,
    Other,
    Unknown,
}

fn kind(expression: &DebugExpression) -> Kind {
    match expression {
        DebugExpression::Integer(_) => Kind::Integer,
        DebugExpression::Boolean(_)
        | DebugExpression::Unary {
            operation: DebugUnaryOperation::Not,
            ..
        } => Kind::Boolean,
        DebugExpression::Unary {
            operation: DebugUnaryOperation::Negate,
            operand,
        } => {
            if matches!(**operand, DebugExpression::Integer(_)) {
                Kind::Integer
            } else {
                Kind::Other
            }
        }
        DebugExpression::Binary { operation, .. } => match operation {
            DebugBinaryOperation::And
            | DebugBinaryOperation::Or
            | DebugBinaryOperation::Xor
            | DebugBinaryOperation::Equal
            | DebugBinaryOperation::NotEqual
            | DebugBinaryOperation::Less
            | DebugBinaryOperation::LessEqual
            | DebugBinaryOperation::Greater
            | DebugBinaryOperation::GreaterEqual
            | DebugBinaryOperation::In => Kind::Boolean,
            DebugBinaryOperation::IntegerDivide | DebugBinaryOperation::Modulo => Kind::Integer,
            _ => Kind::Other,
        },
        DebugExpression::Real(_)
        | DebugExpression::String(_)
        | DebugExpression::Array(_)
        | DebugExpression::Dictionary(_)
        | DebugExpression::Record { .. }
        | DebugExpression::RecordUpdate { .. }
        | DebugExpression::ResultOk(_)
        | DebugExpression::ResultError(_)
        | DebugExpression::OptionSome(_)
        | DebugExpression::OptionNone => Kind::Other,
        _ => Kind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluation::parse_debug_expression;
    use fpas_vm::DebugEvaluationLimits;

    #[test]
    fn logical_watches_reject_non_booleans_including_skipped_operands() {
        for (source, replacement) in [
            ("1 and 2", Some("BitAnd")),
            ("1 or 2", Some("BitOr")),
            ("1 xor 2", Some("BitXor")),
            ("not -1", Some("BitNot")),
            ("false and 1", None),
            ("true or 1", None),
            ("false and (1 + 2)", None),
            ("true or [1]", None),
            ("not 'value'", None),
        ] {
            let error =
                parse_debug_expression(source, DebugEvaluationLimits::default()).expect_err(source);
            assert_eq!(error.code, "evaluation_type", "{source}");
            if let Some(function) = replacement {
                assert!(error.hint.contains(function), "{error:?}");
                assert!(error.hint.contains("uses Std.Bits;"));
            } else {
                assert!(error.hint.contains("Boolean"), "{error:?}");
                assert!(!error.hint.contains("Std.Bits"));
            }
        }
    }
}
