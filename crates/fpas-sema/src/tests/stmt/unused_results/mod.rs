use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNUSED_FUNCTION_RESULT;

mod calls;
mod hints;
mod postfix;

fn unused(source: &str) -> crate::SemaError {
    let errors = check_errors(source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let error = errors.into_iter().next().unwrap();
    assert_eq!(error.code, SEMA_UNUSED_FUNCTION_RESULT, "{error:#?}");
    error
}
