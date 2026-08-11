//! Bounded standalone parsing before safe-IR validation.

use fpas_vm::{DebugEvaluationLimits, DebugExpression};

use super::validate::validate_expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvaluationParseError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) hint: String,
    pub(crate) offset: usize,
    pub(crate) length: usize,
}

pub(crate) fn parse_debug_expression(
    source: &str,
    limits: DebugEvaluationLimits,
) -> Result<DebugExpression, EvaluationParseError> {
    let expression = parse_bounded_ast(source, limits)?;
    validate_expression(&expression, limits)
}

/// Parses one bounded FPAS expression into its source AST.
pub(super) fn parse_bounded_ast(
    source: &str,
    limits: DebugEvaluationLimits,
) -> Result<fpas_parser::Expr, EvaluationParseError> {
    if source.len() > limits.max_expression_bytes {
        return Err(EvaluationParseError {
            code: "evaluation_limit",
            message: format!(
                "debug expression uses {} bytes, exceeding limit {}",
                source.len(),
                limits.max_expression_bytes
            ),
            hint: "Use a shorter watch expression.".to_string(),
            offset: 0,
            length: source.len(),
        });
    }
    preflight_delimiter_depth(source, limits.max_depth)?;
    let (expression, diagnostics) = fpas_parser::parse_expression(source);
    if let Some(diagnostic) = diagnostics.first() {
        let diagnostic = diagnostic.as_diagnostic();
        return Err(EvaluationParseError {
            code: "expression_parse",
            message: diagnostic.message.clone(),
            hint: diagnostic
                .help
                .clone()
                .unwrap_or_else(|| "Use one complete FPAS expression.".to_string()),
            offset: diagnostic.span.offset(),
            length: diagnostic.span.length(),
        });
    }
    Ok(expression)
}

fn preflight_delimiter_depth(source: &str, maximum: usize) -> Result<(), EvaluationParseError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut characters = source.char_indices().peekable();
    while let Some((offset, character)) = characters.next() {
        if character == '\'' {
            if in_string && characters.peek().is_some_and(|(_, next)| *next == '\'') {
                characters.next();
            } else {
                in_string = !in_string;
            }
            continue;
        }
        if in_string {
            continue;
        }
        match character {
            '(' | '[' => {
                depth = depth.saturating_add(1);
                if depth > maximum {
                    return Err(EvaluationParseError {
                        code: "evaluation_limit",
                        message: format!(
                            "debug expression delimiter depth exceeds limit {maximum}"
                        ),
                        hint: "Use a shallower watch expression.".to_string(),
                        offset,
                        length: character.len_utf8(),
                    });
                }
            }
            ')' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}
