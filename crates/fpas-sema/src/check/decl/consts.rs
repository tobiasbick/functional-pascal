//! Constant declaration checking.
//!
//! **Documentation:** `docs/pascal/02-basics.md` (from the repository root).

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_NON_CONSTANT_EXPRESSION};
use fpas_parser::{ConstDef, DesignatorPart, Expr};

impl Checker {
    pub(super) fn check_const_def(&mut self, c: &ConstDef) {
        let declared_ty = self.resolve_type_expr(&c.type_expr);
        let value_ty = self.check_expr(&c.value);
        self.check_type_compat(&declared_ty, &value_ty, "const initializer", c.span);
        if !value_ty.is_error() && !self.const_expr_is_compile_time_known(&c.value) {
            self.error_with_code(
                SEMA_NON_CONSTANT_EXPRESSION,
                format!("Constant `{}` requires a compile-time known initializer", c.name),
                "Use literals, other constants, and pure operators in `const` initializers. Function calls and variables are not allowed.",
                c.span,
            );
        }

        if !self.scopes.define(
            &c.name,
            Symbol {
                ty: declared_ty,
                mutable: false,
                kind: SymbolKind::Const,
            },
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate constant `{}`", c.name),
                "Each name must be unique in the same scope.",
                c.span,
            );
        }
    }

    fn const_expr_is_compile_time_known(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::Integer(..) | Expr::Real(..) | Expr::Str(..) | Expr::Bool(..) => true,
            Expr::Designator(designator) => {
                if !designator
                    .parts
                    .iter()
                    .all(|part| matches!(part, DesignatorPart::Ident(..)))
                {
                    return false;
                }

                let full_name = Self::resolve_designator_name(designator);
                self.ensure_fq_std_unit_loaded(&full_name);

                self.scopes
                    .lookup(&full_name)
                    .or_else(|| {
                        designator.parts.first().and_then(|part| match part {
                            DesignatorPart::Ident(name, _) => self.scopes.lookup(name),
                            DesignatorPart::Index(..) => None,
                        })
                    })
                    .is_some_and(|symbol| {
                        matches!(symbol.kind, SymbolKind::Const | SymbolKind::EnumMember)
                    })
            }
            Expr::UnaryOp { operand, .. } | Expr::Paren(operand, _) => {
                self.const_expr_is_compile_time_known(operand)
            }
            Expr::BinaryOp { left, right, .. } => {
                self.const_expr_is_compile_time_known(left)
                    && self.const_expr_is_compile_time_known(right)
            }
            Expr::ArrayLiteral(elements, _) => elements
                .iter()
                .all(|element| self.const_expr_is_compile_time_known(element)),
            Expr::DictLiteral(pairs, _) => pairs.iter().all(|(key, value)| {
                self.const_expr_is_compile_time_known(key)
                    && self.const_expr_is_compile_time_known(value)
            }),
            Expr::RecordLiteral { fields, .. } => fields
                .iter()
                .all(|field| self.const_expr_is_compile_time_known(&field.value)),
            Expr::ResultOk(inner, _) | Expr::ResultError(inner, _) | Expr::OptionSome(inner, _) => {
                self.const_expr_is_compile_time_known(inner)
            }
            Expr::Try(..) | Expr::Go(..) => false,
            Expr::OptionNone(_) => true,
            Expr::Call { .. } | Expr::Error(_) => false,
            Expr::RecordUpdate { base, fields, .. } => {
                self.const_expr_is_compile_time_known(base)
                    && fields
                        .iter()
                        .all(|f| self.const_expr_is_compile_time_known(&f.value))
            }
        }
    }
}
