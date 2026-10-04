//! Calls through stored record fields and collection elements.
//!
//! **Documentation:** `docs/pascal/language/functions/function-types.md`

use crate::check::Checker;
use crate::types::Ty;
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

impl Checker {
    /// Check a callable projection of a known value root.
    pub(in crate::check) fn try_check_projected_call(
        &mut self,
        call_key: usize,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Option<Ty> {
        let Some((_, consumed)) = self.designator_root_symbol(&designator.parts) else {
            let name = self.resolve_designator_name(designator);
            if let Some(unit) = crate::std_units::missing_std_unit(&name, &self.loaded_std_units) {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                    format!("Unit `{unit}` is not imported"),
                    format!(
                        "Add `{unit}` to the `uses` clause; qualified names do not import a unit."
                    ),
                    span,
                );
                self.check_args_only(args);
                return Some(Ty::Error);
            }
            return None;
        };
        if consumed == designator.parts.len() {
            return None;
        }
        let ty = self.check_designator_expr(designator);
        if ty.is_error() {
            self.check_args_only(args);
            return Some(Ty::Error);
        }
        let name = self.resolve_designator_name(designator);
        Some(self.check_value_call(call_key, &name, &ty, args, span, allow_procedure_result))
    }
}
