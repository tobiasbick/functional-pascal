use super::super::{check_errors, check_ok};

#[test]
fn case_enum_missing_variant_is_non_exhaustive() {
    let errors = check_errors(
        "program T; \
         type Light = enum Red; Yellow; Green; end; \
         begin \
           var L: Light := Light.Red; \
           case L of \
             Light.Red: return; \
             Light.Green: return \
           end \
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
         type Light = enum Red; Yellow; Green; end; \
         begin \
           var L: Light := Light.Red; \
           case L of \
             Light.Red: return \
           else \
             return \
           end \
         end.",
    );
}

#[test]
fn case_result_missing_variant_is_non_exhaustive() {
    let errors = check_errors(
        "program T; \
         begin \
           var R: Result of integer, string := Ok(1); \
           case R of \
             Ok(V): return \
           end \
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
         type Shape = enum Circle(Radius: real); Point; end; \
         begin \
           var S: Shape := Shape.Point; \
           case S of \
             Shape.Circle(R): return \
           end \
         end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE),
        "expected non-exhaustive data-enum case, got: {errors:#?}"
    );
}
