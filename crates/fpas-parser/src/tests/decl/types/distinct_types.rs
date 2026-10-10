//! `distinct` type bodies and the rejected `type X = type Y` form.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use super::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;

fn single_error(src: &str) -> crate::ParseError {
    let (_, diagnostics) = parse_with_errors(src);
    let errors = diagnostics
        .iter()
        .filter_map(ParseDiagnostic::as_parser_error)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1, "{src}: {diagnostics:#?}");
    errors.into_iter().next().unwrap_or_else(|| unreachable!())
}

#[test]
fn distinct_type_bodies_wrap_named_and_qualified_types() {
    let program = parse_ok(
        "program T; type UserId = distinct integer; type Key = DISTINCT Ids.Raw; begin end.",
    );
    let names = program
        .declarations
        .iter()
        .map(|declaration| match declaration {
            Decl::TypeDef(TypeDef {
                body: TypeBody::Distinct(TypeExpr::Named { id, .. }),
                ..
            }) => id.parts.join("."),
            other => panic!("{other:#?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["integer", "Ids.Raw"]);
}

#[test]
fn repeated_type_keyword_is_diagnosed_with_the_canonical_spelling() {
    let error = single_error("program T; type UserId = type integer; begin end.");
    assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
    assert!(error.message.contains("`type X = type Y`"), "{error:#?}");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("`type UserId = distinct integer;`")),
        "{error:#?}"
    );
}

#[test]
fn distinct_outside_a_type_declaration_is_diagnosed() {
    let error = single_error("program T; const X: distinct integer := 1; begin end.");
    assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
    assert!(
        error.message.contains("only valid in a type declaration"),
        "{error:#?}"
    );
}

#[test]
fn call_ranges_are_value_labels_and_single_calls_stay_patterns() {
    let program = parse_ok(
        "program T; begin case Id of when UserId(1)..UserId(9): null; when UserId(10): null; when Shape.Circle(const R): null; else null; end case; end.",
    );
    let Some(Stmt::Case { arms, .. }) = program.body.first() else {
        panic!("{:#?}", program.body);
    };
    assert!(matches!(
        &arms[0].labels[0],
        CaseLabel::Value {
            start: Expr::Call { .. },
            end: Some(Expr::Call { .. }),
            ..
        }
    ));
    let CaseLabel::Pattern(single) = &arms[1].labels[0] else {
        panic!("{:#?}", arms[1].labels);
    };
    assert!(matches!(
        single.conversion_argument(),
        Some(Expr::Integer(10, _))
    ));
    let CaseLabel::Pattern(variant) = &arms[2].labels[0] else {
        panic!("{:#?}", arms[2].labels);
    };
    assert!(variant.conversion_argument().is_none());
}
