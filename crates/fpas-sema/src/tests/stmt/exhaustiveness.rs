use super::super::{check_errors, check_ok};

#[test]
fn case_enum_missing_variant_is_non_exhaustive() {
    let errors = check_errors(
        "program T; \
         type Light = enum Red; Yellow; Green; end enum; \
         begin \
           const L: Light := Light.Red; \
           case L of \
             when Light.Red: return; when \
             Light.Green: return; \
           end case; \
         end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE),
        "expected non-exhaustive case error, got: {errors:#?}"
    );
}

#[test]
fn case_enum_else_branch_skips_exhaustiveness_check() {
    check_ok(
        "program T; \
         type Light = enum Red; Yellow; Green; end enum; \
         begin \
           const L: Light := Light.Red; \
           case L of \
             when Light.Red: return; \
           else \
             return; \
           end case; \
         end.",
    );
}

#[test]
fn case_result_missing_variant_is_non_exhaustive() {
    let errors = check_errors(
        "program T; \
         begin \
           const R: Result of integer, string := Ok(1); \
           case R of \
             when Ok(const V): return; \
           end case; \
         end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE),
        "expected non-exhaustive Result case, got: {errors:#?}"
    );
}

#[test]
fn case_data_enum_missing_variant_is_non_exhaustive() {
    let errors = check_errors(
        "program T; \
         type Shape = enum Circle(Radius: real); Point; end enum; \
         begin \
           const S: Shape := Shape.Point; \
           case S of \
             when Shape.Circle(const R): return; \
           end case; \
         end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE),
        "expected non-exhaustive data-enum case, got: {errors:#?}"
    );
}

#[test]
fn case_on_recursive_enum_binding_is_checked_for_exhaustiveness() {
    let errors = check_errors(
        "program T; \
         type Tree = enum Leaf; Node(Left: Tree; Right: Tree); end enum; \
         begin \
           const T: Tree := Tree.Leaf; \
           case T of \
             when Tree.Node(const L, const R): \
               case L of when \
                 Tree.Node(const A, const B): return; \
               end case; when \
             Tree.Leaf: return; \
           end case; \
         end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE),
        "expected non-exhaustive nested case on recursive binding, got: {errors:#?}"
    );
}

#[test]
fn shadowed_variant_name_does_not_count_toward_exhaustiveness() {
    let errors = check_errors(
        "program T; \
         type Color = enum Red; Green; Blue; end enum; \
         begin \
           const C: Color := Color.Red; \
           const Red: Color := Color.Blue; \
           case C of \
             when Red, Color.Green, Color.Blue: return; \
           end case; \
         end.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE),
        "shadowed Red must leave Color.Red uncovered: {errors:#?}"
    );
}

#[test]
fn qualified_enum_variants_still_satisfy_exhaustiveness() {
    check_ok(
        "program T; \
         type Color = enum Red; Green; Blue; end enum; \
         begin \
           const C: Color := Color.Red; \
           case C of \
             when Color.Red, Color.Green, Color.Blue: return; \
           end case; \
         end.",
    );
}
