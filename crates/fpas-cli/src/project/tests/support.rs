use super::*;

pub(super) fn assert_qualified_designator(parts: &[DesignatorPart], expected: &[&str]) {
    assert_eq!(parts.len(), expected.len());
    for (part, expected_name) in parts.iter().zip(expected.iter()) {
        match part {
            DesignatorPart::Ident(actual, _) => assert_eq!(actual, expected_name),
            other => panic!("expected identifier part, got {other:?}"),
        }
    }
}

pub(super) fn assert_single_ident(parts: &[DesignatorPart], expected: &str) {
    assert_qualified_designator(parts, &[expected]);
}

pub(super) fn assert_named_type(type_expr: &TypeExpr, expected: &[&str]) {
    match type_expr {
        TypeExpr::Named { id, .. } => {
            let actual = id.parts.iter().map(String::as_str).collect::<Vec<_>>();
            assert_eq!(actual, expected);
        }
        other => panic!("expected named type, got {other:?}"),
    }
}
