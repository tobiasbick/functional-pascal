//! Complete existing-language fragments, with no generated declarations in user input.

use crate::SourceQueryError;
use fpas_parser::{Expr, ParamMode};

pub(in crate::refactoring) struct Parameter<'a> {
    pub name: &'a str,
    pub type_text: &'a str,
    pub mode: ParamMode,
    pub argument: &'a str,
}

impl Parameter<'_> {
    pub(in crate::refactoring) fn validate(&self) -> Result<(), SourceQueryError> {
        crate::navigation::validate_identifier(self.name)
            .map_err(|e| SourceQueryError::new("invalid_parameter", e.to_string()))?;
        let (_, types) = fpas_parser::parse_type_expression(self.type_text);
        let (argument, arguments) = fpas_parser::parse_expression(self.argument);
        if !types.is_empty() || !arguments.is_empty() {
            return Err(SourceQueryError::new(
                "invalid_parameter_fragment",
                "Type and argument must each contain exactly one complete existing-language fragment, without trailing tokens or an explicit var marker.",
            ));
        }
        if self.mode == ParamMode::Var && !matches!(argument, Expr::Designator(_)) {
            return Err(SourceQueryError::new(
                "invalid_var_argument",
                "A var argument must be an unmarked storage designator; writable storage is checked in each caller.",
            ));
        }
        Ok(())
    }

    pub(super) fn marker(&self) -> &'static str {
        if self.mode == ParamMode::Var {
            "var "
        } else {
            ""
        }
    }
}

/// A final line comment must not swallow the delimiter supplied by the insertion.
pub(super) fn trailing_newline<'a>(fragment: &str, source: &'a str) -> &'a str {
    let comments = fpas_lexer::collect_comments(fragment);
    let comment_at_end = comments.last().is_some_and(|comment| {
        comment.end_offset().is_some_and(|end| {
            fragment
                .get(end..)
                .is_some_and(|tail| !tail.contains(['\n', '\r']))
        })
    });
    if !comment_at_end {
        ""
    } else if source.contains("\r\n") {
        "\r\n"
    } else if source.contains('\r') {
        "\r"
    } else {
        "\n"
    }
}
