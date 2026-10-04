//! Explicit purity and record-default evaluation regressions.

mod defaults;
mod evaluation;
mod intrinsics;
mod record_calls;
mod recursive_types;
mod signatures;

use crate::tests::check_errors;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

fn rejects(source: &str) {
    let errors = check_errors(source);
    assert!(
        errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH),
        "{errors:#?}"
    );
}
