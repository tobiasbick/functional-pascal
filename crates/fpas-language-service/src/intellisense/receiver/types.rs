//! Canonical type shapes used to filter first-parameter call completions.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use fpas_parser::{Decl, FormalParam, TypeBody, TypeExpr};

/// Check whether a candidate's first parameter accepts the receiver's type shape.
pub(super) fn accepts(expected: &str, actual: &str) -> bool {
    match (parse_type(expected), parse_type(actual)) {
        (Some(expected), Some(actual)) => accepts_type(&expected, &actual),
        _ => expected.eq_ignore_ascii_case(actual),
    }
}

fn parse_type(source: &str) -> Option<TypeExpr> {
    let source =
        format!("program CompletionType; type ValueType = {source}; begin null; end program;");
    let (program, errors) = fpas_parser::parse(&source);
    if !errors.is_empty() {
        return None;
    }
    let Decl::TypeDef(definition) = program.declarations.into_iter().next()? else {
        return None;
    };
    match definition.body {
        TypeBody::Alias(ty) => Some(ty),
        _ => None,
    }
}

fn accepts_type(expected: &TypeExpr, actual: &TypeExpr) -> bool {
    use TypeExpr::*;
    if let Named { id, arguments, .. } = expected
        && arguments.is_empty()
        && id.parts.len() == 1
    {
        let name = &id.parts[0];
        if name.chars().all(|ch| ch.is_ascii_uppercase())
            || name.len() == 1 && name.chars().all(|ch| ch.is_ascii_alphabetic())
        {
            return true;
        }
        if name.eq_ignore_ascii_case("task") && matches!(actual, Task(..)) {
            return true;
        }
    }
    match (expected, actual) {
        (Array(a, _), Array(b, _))
        | (Channel(a, _), Channel(b, _))
        | (Task(a, _), Task(b, _))
        | (Option { inner_type: a, .. }, Option { inner_type: b, .. }) => accepts_type(a, b),
        (
            Dict {
                key_type: a,
                value_type: b,
                ..
            },
            Dict {
                key_type: c,
                value_type: d,
                ..
            },
        )
        | (
            Result {
                ok_type: a,
                err_type: b,
                ..
            },
            Result {
                ok_type: c,
                err_type: d,
                ..
            },
        ) => accepts_type(a, c) && accepts_type(b, d),
        (
            Named {
                id: a,
                arguments: b,
                ..
            },
            Named {
                id: c,
                arguments: d,
                ..
            },
        ) => {
            a.parts
                .last()
                .zip(c.parts.last())
                .is_some_and(|(a, c)| a.eq_ignore_ascii_case(c))
                && b.len() == d.len()
                && b.iter().zip(d).all(|(b, d)| accepts_type(b, d))
        }
        (
            FunctionType {
                params: a,
                return_type: b,
                ..
            },
            FunctionType {
                params: c,
                return_type: d,
                ..
            },
        ) => accepts_parameters(a, c) && accepts_type(b, d),
        (ProcedureType { params: a, .. }, ProcedureType { params: b, .. }) => {
            accepts_parameters(a, b)
        }
        _ => false,
    }
}

fn accepts_parameters(expected: &[FormalParam], actual: &[FormalParam]) -> bool {
    expected.len() == actual.len()
        && expected.iter().zip(actual).all(|(expected, actual)| {
            expected.mutable == actual.mutable
                && accepts_type(&expected.type_expr, &actual.type_expr)
        })
}

#[cfg(test)]
mod tests {
    use super::accepts;

    #[test]
    fn first_parameter_shape_filters_collections() {
        assert!(accepts("array of (T)", "array of (integer)"));
        assert!(!accepts("array of (T)", "string"));
        assert!(accepts("dict of (K, V)", "dict of (string, integer)"));
        assert!(!accepts("dict of (K, V)", "array of (integer)"));
        assert!(!accepts(
            "dict of (string, integer)",
            "dict of (integer, integer)"
        ));
        assert!(!accepts(
            "Result of (string, E)",
            "Result of (integer, string)"
        ));
    }

    #[test]
    fn nested_pairs_and_callable_types_retain_argument_boundaries() {
        assert!(accepts(
            "dict of (string, Result of (T, E))",
            "dict of (string, Result of (integer, string))"
        ));
        assert!(!accepts(
            "dict of (Result of (string, integer), V)",
            "dict of (Result of (string, boolean), integer)"
        ));
        assert!(accepts(
            "Option of (function(A: array of (T)): Result of (T, E))",
            "Option of (function(A: array of (integer)): Result of (integer, string))"
        ));
        assert!(!accepts(
            "Option of (procedure(A: integer))",
            "Option of (procedure(mutable A: integer))"
        ));
        assert!(accepts("task", "task of (integer)"));
    }
}
