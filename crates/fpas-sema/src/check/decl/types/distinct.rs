//! Distinct domain type declarations over scalar underlying types.
//!
//! **Documentation:** `docs/pascal/language/types/distinct-types.md`

use super::Checker;
use crate::types::{DistinctTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{TypeDef, TypeExpr};
use std::sync::Arc;

impl Checker {
    /// Defines a nominal type whose underlying type is `integer`, `real`, `string`, or `boolean`.
    pub(super) fn check_distinct_type_def(&mut self, definition: &TypeDef, underlying: &TypeExpr) {
        let resolved = self.resolve_type_expr(underlying);
        let resolved = self.resolve_visible_type(&resolved);
        let underlying = match resolved {
            Ty::Integer | Ty::Real | Ty::String | Ty::Boolean | Ty::Error => resolved,
            other => {
                let reason = match other {
                    Ty::Distinct(_) => "is already a distinct type",
                    Ty::Record(_) | Ty::Enum(_) => "already has its own type identity",
                    _ => "is not a scalar type",
                };
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Distinct type `{}` cannot be based on `{other}`: `{other}` {reason}",
                        definition.name
                    ),
                    "The underlying type of a distinct type must be `integer`, `real`, `string`, or `boolean`, for example `type UserId = distinct integer;`. Use a record for structured domain values.",
                    definition.span,
                );
                Ty::Error
            }
        };
        let owner_unit = self
            .scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.clone());
        self.define_type_symbol(
            definition,
            Ty::Distinct(Arc::new(DistinctTy {
                name: definition.name.clone(),
                owner_unit,
                underlying,
            })),
        );
    }
}
