//! Accepted debugger-expression shapes for task-handle assignment.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::super::super::evaluation::{DebugEvaluationLimits, DebugExpression};
use super::super::super::types::{DebugErrorKind, DebugSessionError};

/// Extract exactly one simple binding name after parentheses are already removed.
pub(in crate::vm::debug) fn extract(
    expression: &DebugExpression,
    limits: DebugEvaluationLimits,
) -> Result<String, DebugSessionError> {
    if limits.max_operations == 0 {
        return Err(source_limit(
            "debug task source exceeds operation limit 0",
            "Allow at least one evaluation operation for the task source binding.",
        ));
    }
    let DebugExpression::Name(name) = expression else {
        return Err(unsupported_source());
    };
    if !is_identifier(name) {
        return Err(unsupported_source());
    }
    if name.len() > limits.max_expression_bytes {
        return Err(source_limit(
            &format!(
                "debug task source exceeds byte limit {}",
                limits.max_expression_bytes
            ),
            "Use a shorter binding name, or raise the expression byte limit.",
        ));
    }
    Ok(name.clone())
}

fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn source_limit(message: &str, hint: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::EvaluationLimit,
        message: message.to_string(),
        hint: hint.to_string(),
    }
}

fn unsupported_source() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: "debug task assignment requires a binding name".to_string(),
        hint: "Copy an existing task handle such as `Current := Pending`. Do not enter a numeric ID or `<task N>` display text."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::extract;
    use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
    use crate::vm::debug::types::DebugErrorKind;

    #[test]
    fn simple_binding_names_are_accepted() {
        assert_eq!(
            extract(
                &DebugExpression::Name("Pending".to_string()),
                DebugEvaluationLimits::default()
            )
            .expect("simple"),
            "Pending"
        );
    }

    #[test]
    fn literals_calls_selectors_and_display_text_are_rejected() {
        let invalid = [
            DebugExpression::Integer(1),
            DebugExpression::String("<task 1>".to_string()),
            DebugExpression::Name("9Bad".to_string()),
            DebugExpression::Name("<task 1>".to_string()),
            DebugExpression::Callable("Seven".to_string()),
            DebugExpression::Field {
                base: Box::new(DebugExpression::Name("Box".to_string())),
                name: "Job".to_string(),
            },
            DebugExpression::Call {
                callee: Box::new(DebugExpression::Name("Pending".to_string())),
                arguments: Vec::new(),
            },
        ];
        for expression in invalid {
            let error = extract(&expression, DebugEvaluationLimits::default())
                .expect_err("unsupported source");
            assert_eq!(error.kind, DebugErrorKind::VariableValueType);
            assert!(error.hint.contains("Current := Pending"), "{}", error.hint);
            assert!(error.hint.contains("Do not enter"), "{}", error.hint);
        }
    }

    #[test]
    fn over_limit_binding_names_are_rejected() {
        let operation = extract(
            &DebugExpression::Name("Pending".to_string()),
            DebugEvaluationLimits {
                max_operations: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("operation limit");
        assert_eq!(operation.kind, DebugErrorKind::EvaluationLimit);
        let bytes = extract(
            &DebugExpression::Name("Pending".to_string()),
            DebugEvaluationLimits {
                max_expression_bytes: 3,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("byte limit");
        assert_eq!(bytes.kind, DebugErrorKind::EvaluationLimit);
    }
}
