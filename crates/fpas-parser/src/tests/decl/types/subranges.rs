//! Range declaration grammar and recovery boundaries.

use super::*;

#[test]
fn subrange_bounds_accept_static_expression_forms_without_consuming_later_declarations() {
    let program = parse_ok(
        "program T; type Signed = (-Limit)..Limit + 1;
        type Conditional = if true then 0 else -1 end if..100;
        type Selected = case 1 of when 1: 0; else -1; end case..100;
        type Box = record Value: integer := case 1 of when 0..2: 1; else 2; end case; end record;
        type Alias = integer; begin end.",
    );
    assert_eq!(program.declarations.len(), 5);
    assert!(program.declarations[..3].iter().all(|declaration| matches!(
        declaration,
        Decl::TypeDef(TypeDef {
            body: TypeBody::Subrange { .. },
            ..
        })
    )));
    assert!(matches!(
        &program.declarations[3],
        Decl::TypeDef(TypeDef {
            body: TypeBody::Record(_),
            ..
        })
    ));
}

#[test]
fn subrange_base_annotations_and_missing_bounds_are_rejected_with_recovery() {
    for body in [
        "type R = integer range 0..100;",
        "type R = 0..100 of integer;",
        "type R = 0..;",
        "type R of T = 0..100;",
    ] {
        let (_, errors) = parse_with_errors(&format!(
            "program T; {body} type Keep = integer; begin end."
        ));
        assert!(!errors.is_empty(), "accepted {body}");
    }
}
