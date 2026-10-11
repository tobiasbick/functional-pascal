//! Static integer bounds resolved during whole-unit type collection.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

use super::Checker;
use crate::types::{SubrangeTy, Ty};
use fpas_diagnostics::codes::{SEMA_NON_CONSTANT_EXPRESSION, SEMA_TYPE_MISMATCH};
use fpas_parser::{Decl, Expr, TypeBody, TypeDef};
use fpas_unit::interface::ConstantValue;
use std::sync::Arc;

impl Checker {
    /// Checks inclusive bounds using preceding constants and imported static values.
    pub(super) fn check_subrange_type_def(&mut self, definition: &TypeDef) {
        let retained = Arc::new(Decl::TypeDef(definition.clone()));
        self.type_collection
            .static_bound_declarations
            .push(Arc::clone(&retained));
        let Decl::TypeDef(TypeDef {
            body: TypeBody::Subrange { lower, upper },
            ..
        }) = retained.as_ref()
        else {
            return;
        };
        let constants = self
            .type_collection
            .static_bound_declarations
            .iter()
            .filter(|declaration| {
                matches!(declaration.as_ref(), Decl::Const(constant) if constant.span.offset < definition.span.offset)
            })
            .map(Arc::clone)
            .collect::<Vec<_>>();
        self.scopes.push_scope();
        // Reuse ordered constant classification without publishing temporary value names.
        for declaration in &constants {
            let Decl::Const(constant) = declaration.as_ref() else {
                continue;
            };
            let count = self.errors.len();
            self.check_binding(
                &constant.name,
                Some(&constant.type_expr),
                &constant.value,
                constant.span,
                false,
                true,
            );
            self.errors.truncate(count);
        }
        let lower = self.check_subrange_bound(lower);
        let upper = self.check_subrange_bound(upper);
        self.scopes.pop_scope();
        let ty = match (lower, upper) {
            (Some(lower), Some(upper)) if lower <= upper => Ty::Subrange(SubrangeTy {
                name: definition.name.clone(),
                lower,
                upper,
            }),
            (Some(lower), Some(upper)) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Subrange `{}` has inverted bounds {lower}..{upper}",
                        definition.name
                    ),
                    "Write the lower bound first, for example `type Percent = 0..100;`.",
                    definition.span,
                );
                Ty::Error
            }
            _ => Ty::Error,
        };
        self.define_type_symbol(definition, ty);
    }

    fn check_subrange_bound(&mut self, expression: &Expr) -> Option<i64> {
        let before = self.errors.len();
        let ty = self.check_expr(expression);
        if let Some((name, span)) = self.non_constant_part(expression) {
            self.errors.truncate(before);
            self.error_with_code(SEMA_NON_CONSTANT_EXPRESSION, format!("Subrange bound requires a compile-time constant; {name} is computed at runtime"),
                "Use an integer literal or static constant expression; calls and computed constants cannot define bounds.", span);
            return None;
        }
        if ty.is_error() {
            return None;
        }
        if !Ty::Integer.assignment_compatible_with(&ty) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Subrange bound must be an integer, found `{ty}`"),
                "Use integer compile-time bounds, for example `type Percent = 0..100;`.",
                expression.span(),
            );
            return None;
        }
        if let Some(ConstantValue::Integer(value)) = self.scalar_constant_value(expression) {
            return Some(value);
        }
        self.error_with_code(
            SEMA_NON_CONSTANT_EXPRESSION,
            "Subrange bound has no valid compile-time integer value",
            "Use a static integer expression that evaluates successfully.",
            expression.span(),
        );
        None
    }
}
