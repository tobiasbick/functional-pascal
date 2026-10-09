//! Call argument validation. See `docs/pascal/language/functions/parameters.md`.

use fpas_parser::Expr;
use fpas_vm::DebugExpression;

use super::parse::EvaluationParseError;

/// Lowers fully positional or fully named arguments in their written order.
pub(super) fn lower(
    arguments: &[Expr],
    mut lower_value: impl FnMut(&Expr) -> Result<DebugExpression, EvaluationParseError>,
) -> Result<Vec<DebugExpression>, EvaluationParseError> {
    let named = arguments
        .iter()
        .any(|argument| argument.argument_name().is_some());
    let mut names = std::collections::HashSet::new();
    arguments
        .iter()
        .map(|argument| {
            if let Expr::NamedArgument {
                name,
                name_span,
                value,
                ..
            } = argument
            {
                if !names.insert(name.to_ascii_lowercase()) {
                    return Err(EvaluationParseError {
                        code: "evaluation_type",
                        message: format!("debug call names argument `{name}` more than once"),
                        hint: "Pass each declared parameter or field exactly once.".into(),
                        offset: name_span.offset,
                        length: name_span.length,
                    });
                }
                return Ok(DebugExpression::NamedArgument {
                    name: name.clone(),
                    value: Box::new(lower_value(value)?),
                });
            }
            if named {
                let span = argument.span();
                return Err(EvaluationParseError {
                    code: "evaluation_type",
                    message: "debug call mixes positional and named arguments".into(),
                    hint: "Use either all positional arguments or all named arguments.".into(),
                    offset: span.offset,
                    length: span.length,
                });
            }
            lower_value(argument)
        })
        .collect()
}
