//! Bounded parsing and lowering of textual debugger assignment targets.

use fpas_parser::Expr;
use fpas_vm::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugEvaluationLimits, DebugExpression,
};

use super::parse::{EvaluationParseError, parse_bounded_ast};
use super::validate::validate_expression;

/// Parses one bounded textual assignment target for debugger mutation.
pub(crate) fn parse_debug_assignment_target(
    source: &str,
    limits: DebugEvaluationLimits,
) -> Result<DebugAssignmentTarget, EvaluationParseError> {
    let expression = parse_bounded_ast(source, limits).map_err(|mut error| {
        if error.code == "expression_parse" {
            error.code = "expression_target_parse";
            error.hint =
                "Use one complete target such as `Counter`, `Origin.X`, or `Items[Index]`."
                    .to_string();
        }
        error
    })?;
    if !matches!(expression, Expr::Designator(_)) {
        let span = expression.span();
        return Err(EvaluationParseError {
            code: "expression_target_unsupported",
            message: "debugger assignment target must be a named stored-value designator"
                .to_string(),
            hint: "Use a target such as `Counter`, `Origin.X`, or `Items[Index]`.".to_string(),
            offset: span.offset,
            length: span.length,
        });
    }
    let lowered = validate_expression(&expression, limits)?;
    let mut selectors = Vec::new();
    let root = collect_target(lowered, &mut selectors).map_err(|()| {
        let span = expression.span();
        EvaluationParseError {
            code: "expression_target_unsupported",
            message: "debugger assignment target contains an unsupported computed value"
                .to_string(),
            hint: "Start with a visible binding and use only stored fields or indexes.".to_string(),
            offset: span.offset,
            length: span.length,
        }
    })?;
    Ok(DebugAssignmentTarget { root, selectors })
}

fn collect_target(
    expression: DebugExpression,
    selectors: &mut Vec<DebugAssignmentSelector>,
) -> Result<String, ()> {
    match expression {
        DebugExpression::Name(name) => Ok(name),
        DebugExpression::Field { base, name } => {
            let root = collect_target(*base, selectors)?;
            selectors.push(DebugAssignmentSelector::Field(name));
            Ok(root)
        }
        DebugExpression::Index { base, index } => {
            let root = collect_target(*base, selectors)?;
            selectors.push(DebugAssignmentSelector::Index(*index));
            Ok(root)
        }
        _ => Err(()),
    }
}
