use super::super::{check_errors, check_ok};
use crate::{Ty, analyze_with_types, expr_lookup_key};

#[test]
fn ref_self_referential_record_type_is_valid() {
    check_ok(
        "program T; \
         type Node = record Value: integer; Parent: Option of ref Node; end; \
         var Root: ref Node := new Node with Value := 1; Parent := None; end; \
         begin end.",
    );
}

#[test]
fn new_expression_records_ref_type() {
    let (program, parse_errors) = fpas_parser::parse(
        "program T; \
         type Point = record X: integer; end; \
         var P: ref Point := new Point with X := 1; end; \
         begin end.",
    );
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");

    let value = match &program.declarations[1] {
        fpas_parser::Decl::Var(var_def) => &var_def.value,
        other => panic!("expected variable declaration, got {other:?}"),
    };

    let (errors, types, _, _, _, _) = analyze_with_types(&program);
    assert!(errors.is_empty(), "{errors:#?}");

    let key = expr_lookup_key(value);
    assert!(matches!(
        types.get(&key),
        Some(Ty::Ref(inner)) if matches!(inner.as_ref(), Ty::Record(record) if record.name == "Point")
    ));
}

#[test]
fn ref_requires_record_type() {
    let errors = check_errors(
        "program T; \
         type ScalarRef = ref integer; \
         begin end.",
    );
    assert!(errors.iter().any(|error| {
        error.code == fpas_diagnostics::codes::SEMA_TYPE_MISMATCH
            && error.message.contains("`ref` requires a record type")
    }));
}

#[test]
fn mutable_ref_field_assignment_is_allowed() {
    check_ok(
        "program T; \
         type Point = record X: integer; end; \
         begin \
           mutable var P: ref Point := new Point with X := 1; end; \
           P.X := 2 \
         end.",
    );
}

#[test]
fn immutable_ref_field_assignment_is_rejected() {
    let errors = check_errors(
        "program T; \
         type Point = record X: integer; end; \
         begin \
           var P: ref Point := new Point with X := 1; end; \
           P.X := 2 \
         end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT })
    );
}

#[test]
fn const_ref_initializer_is_rejected_as_non_constant() {
    let errors = check_errors(
        "program T; \
         type Point = record X: integer; end; \
         const P: ref Point := new Point with X := 1; end; \
         begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| { error.code == fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION })
    );
}
