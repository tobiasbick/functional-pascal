//! Bounded parsing and lowering of textual debugger assignment targets.

use fpas_lexer::Token;
use fpas_parser::{Expr, ParseDiagnostic};
use fpas_vm::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugEvaluationLimits, DebugExpression,
};

use super::parse::{EvaluationParseError, check_bounded_source, reject_diagnostics};
use super::validate::validate_expression;

/// Parses one bounded textual assignment target for debugger mutation.
pub(crate) fn parse_debug_assignment_target(
    source: &str,
    limits: DebugEvaluationLimits,
) -> Result<DebugAssignmentTarget, EvaluationParseError> {
    let expression = parse_target_ast(source, limits).map_err(|mut error| {
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

fn parse_target_ast(
    source: &str,
    limits: DebugEvaluationLimits,
) -> Result<Expr, EvaluationParseError> {
    check_bounded_source(source, limits)?;
    let (mut tokens, lexer_diagnostics) = fpas_lexer::lex(source);
    let mut after_dot = false;
    let mut delimiter_depth = 0_usize;
    for token in &mut tokens {
        if after_dot {
            let constructor = match token.token {
                Token::Ok => Some("Ok"),
                Token::Error => Some("Error"),
                Token::Some => Some("Some"),
                Token::None => Some("None"),
                _ => None,
            };
            if let Some(constructor) = constructor {
                let spelling = token
                    .span
                    .text(source)
                    .map(str::to_owned)
                    .unwrap_or_else(|| constructor.to_string());
                token.token = Token::Ident(spelling);
            }
        }
        match token.token {
            Token::LParen | Token::LBracket => {
                delimiter_depth = delimiter_depth.saturating_add(1);
            }
            Token::RParen | Token::RBracket => {
                delimiter_depth = delimiter_depth.saturating_sub(1);
            }
            _ => {}
        }
        after_dot = delimiter_depth == 0 && matches!(token.token, Token::Dot);
    }
    let (expression, parser_diagnostics) = fpas_parser::parse_tokens_expression(tokens);
    let diagnostics = lexer_diagnostics
        .into_iter()
        .map(ParseDiagnostic::Lexer)
        .chain(parser_diagnostics)
        .collect();
    reject_diagnostics(expression, diagnostics)
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
