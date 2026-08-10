//! Bounded logpoint template parsing with FPAS expression interpolation.

use fpas_vm::{DebugEvaluationLimits, DebugExpression};

use super::{EvaluationParseError, parse_debug_expression};

/// Resource limits for one configured logpoint template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LogMessageLimits {
    pub(crate) max_template_bytes: usize,
    pub(crate) max_interpolations: usize,
    pub(crate) max_session_output_bytes: usize,
}

impl Default for LogMessageLimits {
    fn default() -> Self {
        Self {
            max_template_bytes: 16_384,
            max_interpolations: 64,
            max_session_output_bytes: 1_048_576,
        }
    }
}

/// Validated logpoint template with pre-parsed read-only expressions.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LogMessage {
    segments: Vec<LogSegment>,
}

impl LogMessage {
    pub(crate) fn parse(
        source: &str,
        log_limits: LogMessageLimits,
        evaluation_limits: DebugEvaluationLimits,
    ) -> Result<Self, EvaluationParseError> {
        if source.len() > log_limits.max_template_bytes {
            return Err(template_error(
                "log_message_limit",
                format!(
                    "logpoint template uses {} bytes, exceeding limit {}",
                    source.len(),
                    log_limits.max_template_bytes
                ),
                "Use a shorter logpoint message.",
                0,
                source.len(),
            ));
        }
        let mut segments = Vec::new();
        let mut text = String::new();
        let mut interpolations = 0_usize;
        let bytes = source.as_bytes();
        let mut index = 0_usize;
        while index < bytes.len() {
            match bytes[index] {
                b'{' if bytes.get(index + 1) == Some(&b'{') => {
                    text.push('{');
                    index += 2;
                }
                b'}' if bytes.get(index + 1) == Some(&b'}') => {
                    text.push('}');
                    index += 2;
                }
                b'{' => {
                    if !text.is_empty() {
                        segments.push(LogSegment::Text(std::mem::take(&mut text)));
                    }
                    let start = index;
                    index += 1;
                    let expression_start = index;
                    while index < bytes.len() && bytes[index] != b'}' {
                        if bytes[index] == b'{' {
                            return Err(template_error(
                                "invalid_log_message",
                                "nested logpoint interpolation braces are unsupported",
                                "Close the current `{expression}` before starting another.",
                                index,
                                1,
                            ));
                        }
                        index += 1;
                    }
                    if index == bytes.len() {
                        return Err(template_error(
                            "invalid_log_message",
                            "logpoint interpolation has no closing `}`",
                            "Close every `{expression}` interpolation.",
                            start,
                            source.len().saturating_sub(start),
                        ));
                    }
                    let expression_source = &source[expression_start..index];
                    if expression_source.trim().is_empty() {
                        return Err(template_error(
                            "invalid_log_message",
                            "logpoint interpolation expression is empty",
                            "Put one read-only FPAS expression between the braces.",
                            expression_start,
                            index.saturating_sub(expression_start),
                        ));
                    }
                    interpolations = interpolations.saturating_add(1);
                    if interpolations > log_limits.max_interpolations {
                        return Err(template_error(
                            "log_message_limit",
                            format!(
                                "logpoint interpolation count exceeds limit {}",
                                log_limits.max_interpolations
                            ),
                            "Use fewer interpolated expressions.",
                            start,
                            index.saturating_sub(start).saturating_add(1),
                        ));
                    }
                    let expression = parse_debug_expression(expression_source, evaluation_limits)
                        .map_err(|mut error| {
                        error.offset = error.offset.saturating_add(expression_start);
                        error
                    })?;
                    segments.push(LogSegment::Expression(expression));
                    index += 1;
                }
                b'}' => {
                    return Err(template_error(
                        "invalid_log_message",
                        "logpoint template contains an unmatched `}`",
                        "Escape a literal closing brace as `}}`.",
                        index,
                        1,
                    ));
                }
                _ => {
                    let Some(character) = source[index..].chars().next() else {
                        return Err(template_error(
                            "invalid_log_message",
                            "logpoint template ended unexpectedly",
                            "Use complete UTF-8 text and balanced braces.",
                            index,
                            0,
                        ));
                    };
                    text.push(character);
                    index += character.len_utf8();
                }
            }
        }
        if !text.is_empty() {
            segments.push(LogSegment::Text(text));
        }
        Ok(Self { segments })
    }

    pub(crate) fn segments(&self) -> &[LogSegment] {
        &self.segments
    }
}

/// One literal or expression template segment.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LogSegment {
    Text(String),
    Expression(DebugExpression),
}

fn template_error(
    code: &'static str,
    message: impl Into<String>,
    hint: impl Into<String>,
    offset: usize,
    length: usize,
) -> EvaluationParseError {
    EvaluationParseError {
        code,
        message: message.into(),
        hint: hint.into(),
        offset,
        length,
    }
}
