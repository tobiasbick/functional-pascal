//! Accepted debugger-expression shapes for function-value assignment.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::super::super::evaluation::{DebugEvaluationLimits, DebugExpression};
use super::super::super::types::{DebugErrorKind, DebugSessionError};

/// One supported function-assignment source after parentheses are removed.
#[derive(Debug, Clone, PartialEq)]
pub(in crate::vm::debug) enum FunctionSource {
    /// A simple identifier: try a visible binding first, then the routine catalog.
    BindingOrRoutine(String),
    /// An identifier-only qualified chain resolved only through the routine catalog.
    Routine(String),
    /// A receiver expression plus one method member, with an optional catalog fallback.
    BoundReceiver {
        /// Expression evaluated once before mutation preparation.
        receiver: Box<DebugExpression>,
        /// Source method member name.
        member: String,
        /// Identifier-only full spelling when catalog fallback is possible.
        catalog_name: Option<String>,
    },
}

impl FunctionSource {
    /// Requested source spelling used in diagnostics.
    pub(in crate::vm::debug) fn requested(&self) -> &str {
        match self {
            Self::BindingOrRoutine(name) | Self::Routine(name) => name,
            Self::BoundReceiver {
                member,
                catalog_name,
                ..
            } => catalog_name.as_deref().unwrap_or(member),
        }
    }

    /// Expression that must be evaluated before replacement preparation.
    pub(in crate::vm::debug) fn evaluation_expression(&self) -> DebugExpression {
        match self {
            Self::BindingOrRoutine(name) | Self::Routine(name) => {
                DebugExpression::Name(name.clone())
            }
            Self::BoundReceiver { receiver, .. } => receiver.as_ref().clone(),
        }
    }

    /// Whether an unknown receiver name may be interpreted as a catalog routine.
    pub(in crate::vm::debug) fn allows_catalog_fallback(&self) -> bool {
        match self {
            Self::BindingOrRoutine(_) | Self::Routine(_) => true,
            Self::BoundReceiver { catalog_name, .. } => catalog_name.is_some(),
        }
    }
}

/// Extract a simple name or identifier-only qualified routine chain.
pub(in crate::vm::debug) fn extract(
    expression: &DebugExpression,
    limits: DebugEvaluationLimits,
) -> Result<FunctionSource, DebugSessionError> {
    if let DebugExpression::Field { base, name } = expression {
        if !is_identifier(name) {
            return Err(unsupported_source());
        }
        let catalog_name = identifier_chain(expression, limits)?.map(|parts| parts.join("."));
        return Ok(FunctionSource::BoundReceiver {
            receiver: base.clone(),
            member: name.clone(),
            catalog_name,
        });
    }
    let Some(parts) = identifier_chain(expression, limits)? else {
        return Err(unsupported_source());
    };
    match parts.as_slice() {
        [] => Err(unsupported_source()),
        [name] => Ok(FunctionSource::BindingOrRoutine((*name).to_string())),
        _ => Ok(FunctionSource::Routine(parts.join("."))),
    }
}

fn identifier_chain(
    expression: &DebugExpression,
    limits: DebugEvaluationLimits,
) -> Result<Option<Vec<&str>>, DebugSessionError> {
    if limits.max_operations == 0 {
        return Err(source_limit(
            "debug function source exceeds operation limit 0",
            "Allow at least one evaluation operation for the routine source.",
        ));
    }
    let mut current = expression;
    let mut suffix = Vec::new();
    while let DebugExpression::Field { base, name } = current {
        if suffix.len() >= limits.max_depth {
            return Err(source_limit(
                &format!(
                    "debug function source exceeds depth limit {}",
                    limits.max_depth
                ),
                "Use a shorter qualified routine name, or raise the evaluation depth limit.",
            ));
        }
        if !is_identifier(name) {
            return Ok(None);
        }
        suffix.push(name.as_str());
        current = base;
    }
    let mut parts = match current {
        DebugExpression::Name(name) if is_identifier(name) => vec![name.as_str()],
        DebugExpression::Callable(name) => {
            if name.len() > limits.max_expression_bytes {
                return Err(source_limit(
                    &format!(
                        "debug function source exceeds byte limit {}",
                        limits.max_expression_bytes
                    ),
                    "Use a shorter routine name, or raise the expression byte limit.",
                ));
            }
            let parts = name.split('.').collect::<Vec<_>>();
            if parts.iter().all(|part| is_identifier(part)) {
                parts
            } else {
                return Ok(None);
            }
        }
        DebugExpression::Name(_) => return Ok(None),
        _ => return Ok(None),
    };
    suffix.reverse();
    parts.extend(suffix);
    let depth = parts.len().saturating_sub(1);
    if depth > limits.max_depth {
        return Err(source_limit(
            &format!(
                "debug function source exceeds depth limit {}",
                limits.max_depth
            ),
            "Use a shorter qualified routine name, or raise the evaluation depth limit.",
        ));
    }
    let bytes = parts
        .iter()
        .map(|part| part.len())
        .sum::<usize>()
        .saturating_add(parts.len().saturating_sub(1));
    if bytes > limits.max_expression_bytes {
        return Err(source_limit(
            &format!(
                "debug function source exceeds byte limit {}",
                limits.max_expression_bytes
            ),
            "Use a shorter routine name, or raise the expression byte limit.",
        ));
    }
    Ok(Some(parts))
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
        message: "debug function assignment requires a binding name or a unique executable routine name"
            .to_string(),
        hint: "Copy an existing function value such as `Current := Backup`, or assign a routine such as `Current := AddTwo` or a capturing nested routine from its owner frame."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{FunctionSource, extract};
    use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};

    #[test]
    fn simple_and_qualified_identifier_chains_are_accepted() {
        assert_eq!(
            extract(
                &DebugExpression::Name("AddTwo".to_string()),
                DebugEvaluationLimits::default()
            )
            .expect("simple"),
            FunctionSource::BindingOrRoutine("AddTwo".to_string())
        );
        assert_eq!(
            extract(
                &DebugExpression::Callable("Math.Transform".to_string()),
                DebugEvaluationLimits::default()
            )
            .expect("callable"),
            FunctionSource::Routine("Math.Transform".to_string())
        );
        let field = extract(
            &DebugExpression::Field {
                base: Box::new(DebugExpression::Name("Math".to_string())),
                name: "Transform".to_string(),
            },
            DebugEvaluationLimits::default(),
        )
        .expect("field chain");
        assert_eq!(field.requested(), "Math.Transform");
        assert!(field.allows_catalog_fallback());
        assert!(
            extract(
                &DebugExpression::Call {
                    callee: Box::new(DebugExpression::Name("AddTwo".to_string())),
                    arguments: Vec::new(),
                },
                DebugEvaluationLimits::default()
            )
            .is_err()
        );
    }

    #[test]
    fn invalid_and_over_limit_identifier_chains_are_rejected() {
        let invalid = [
            DebugExpression::Name("9Bad".to_string()),
            DebugExpression::Callable("Math..Transform".to_string()),
            DebugExpression::Field {
                base: Box::new(DebugExpression::Name("Math".to_string())),
                name: "not-valid".to_string(),
            },
        ];
        for expression in invalid {
            assert_eq!(
                extract(&expression, DebugEvaluationLimits::default())
                    .expect_err("invalid identifier")
                    .kind,
                crate::vm::debug::types::DebugErrorKind::VariableValueType
            );
        }

        let qualified = DebugExpression::Field {
            base: Box::new(DebugExpression::Name("Math".to_string())),
            name: "Transform".to_string(),
        };
        let operation = extract(
            &qualified,
            DebugEvaluationLimits {
                max_operations: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("operation limit");
        assert_eq!(
            operation.kind,
            crate::vm::debug::types::DebugErrorKind::EvaluationLimit
        );
        let depth = extract(
            &qualified,
            DebugEvaluationLimits {
                max_depth: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("depth limit");
        assert_eq!(
            depth.kind,
            crate::vm::debug::types::DebugErrorKind::EvaluationLimit
        );
        let bytes = extract(
            &qualified,
            DebugEvaluationLimits {
                max_expression_bytes: 4,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("byte limit");
        assert_eq!(
            bytes.kind,
            crate::vm::debug::types::DebugErrorKind::EvaluationLimit
        );
        let callable_bytes = extract(
            &DebugExpression::Callable("Math.Transform".to_string()),
            DebugEvaluationLimits {
                max_expression_bytes: 4,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("callable byte limit");
        assert_eq!(
            callable_bytes.kind,
            crate::vm::debug::types::DebugErrorKind::EvaluationLimit
        );
        let callable_depth = extract(
            &DebugExpression::Callable("Math.Transform".to_string()),
            DebugEvaluationLimits {
                max_depth: 0,
                ..DebugEvaluationLimits::default()
            },
        )
        .expect_err("callable depth limit");
        assert_eq!(
            callable_depth.kind,
            crate::vm::debug::types::DebugErrorKind::EvaluationLimit
        );
    }
}
