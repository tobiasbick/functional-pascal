use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::{Designator, DesignatorPart};

impl Checker {
    pub(crate) fn check_designator_expr(&mut self, designator: &Designator) -> Ty {
        let only_ident_chain = designator
            .parts
            .iter()
            .all(|p| matches!(p, DesignatorPart::Ident(_, _)));

        if only_ident_chain {
            let full_name = Self::resolve_designator_name(designator);
            self.ensure_fq_std_unit_loaded(&full_name);
            if let Some(symbol) = self.scopes.lookup(&full_name) {
                return symbol.ty.clone();
            }
        }
        self.check_designator_path(designator)
    }

    fn check_designator_path(&mut self, designator: &Designator) -> Ty {
        if designator.parts.is_empty() {
            return Ty::Error;
        }

        match &designator.parts[0] {
            DesignatorPart::Index(_, span) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "Expression cannot start with an index",
                    "Use a variable or constant name first.",
                    *span,
                );
                Ty::Error
            }
            DesignatorPart::Ident(first, _) => {
                let Some(symbol) = self.scopes.lookup(first) else {
                    let full_name = Self::resolve_designator_name(designator);
                    let is_qualified_ident_chain = designator.parts.len() > 1
                        && designator
                            .parts
                            .iter()
                            .all(|part| matches!(part, DesignatorPart::Ident(_, _)));

                    let hint = if let Some(ambiguous_hint) = self.ambiguous_hint(first) {
                        ambiguous_hint
                    } else if is_qualified_ident_chain {
                        if crate::std_units::looks_like_std_qualified_name(&full_name) {
                            self.hint_unknown_callable(&full_name)
                        } else {
                            "Check that the unit is listed in `uses` and that the symbol is public. Private unit members are not visible outside their unit.".to_string()
                        }
                    } else if crate::std_units::looks_like_std_qualified_name(first) {
                        self.hint_unknown_callable(first)
                    } else {
                        "Check spelling or declare the variable or constant.".to_string()
                    };

                    let message = if self.ambiguous_imports.contains_key(first) {
                        format!("Ambiguous name `{first}`")
                    } else if is_qualified_ident_chain {
                        format!("Undefined identifier `{full_name}`")
                    } else {
                        format!("Undefined identifier `{first}`")
                    };

                    self.error_with_code(SEMA_UNKNOWN_NAME, message, hint, designator.span);
                    return Ty::Error;
                };

                let mut ty = symbol.ty.clone();
                for part in &designator.parts[1..] {
                    ty = self.resolve_visible_type(&ty);
                    if let Ty::Ref(inner) = ty {
                        ty = self.resolve_visible_type(inner.as_ref());
                    }

                    ty = match part {
                        DesignatorPart::Ident(field, span) => match &ty {
                            Ty::Record(record_ty) => {
                                if let Some((_, field_ty)) =
                                    record_ty.fields.iter().find(|(name, _)| name == field)
                                {
                                    field_ty.clone()
                                } else {
                                    self.error_with_code(
                                        SEMA_UNKNOWN_NAME,
                                        format!("Record has no field `{field}`"),
                                        "Check the field name against the record type.",
                                        *span,
                                    );
                                    Ty::Error
                                }
                            }
                            _ => {
                                self.error_with_code(
                                    SEMA_TYPE_MISMATCH,
                                    format!("`.{field}` requires a record value"),
                                    "Only records support field access with `.`.",
                                    *span,
                                );
                                Ty::Error
                            }
                        },
                        DesignatorPart::Index(index_expr, span) => {
                            let index_ty = self.check_expr(index_expr);

                            match &ty {
                                Ty::Array(inner) => {
                                    if index_ty != Ty::Integer && !index_ty.is_error() {
                                        self.error_with_code(
                                            SEMA_TYPE_MISMATCH,
                                            "Array index must be integer",
                                            "Use an integer index expression.",
                                            super::super::spans::expr_span(index_expr),
                                        );
                                    }
                                    *inner.clone()
                                }
                                Ty::Dict(key_ty, val_ty) => {
                                    if !index_ty.compatible_with(key_ty) && !index_ty.is_error() {
                                        self.error_with_code(
                                            SEMA_TYPE_MISMATCH,
                                            format!(
                                                "Dict key type mismatch: expected `{key_ty}`, got `{index_ty}`"
                                            ),
                                            "Use a key matching the dict's key type.",
                                            super::super::spans::expr_span(index_expr),
                                        );
                                    }
                                    *val_ty.clone()
                                }
                                Ty::String => {
                                    if index_ty != Ty::Integer && !index_ty.is_error() {
                                        self.error_with_code(
                                            SEMA_TYPE_MISMATCH,
                                            "String index must be an integer",
                                            "Use an integer index, e.g. S[0].",
                                            super::super::spans::expr_span(index_expr),
                                        );
                                    }
                                    Ty::Char
                                }
                                _ => {
                                    self.error_with_code(
                                        SEMA_TYPE_MISMATCH,
                                        "Indexed value is not an array, dict, or string",
                                        "Use `A[I]` only on array, dict, or string values.",
                                        *span,
                                    );
                                    Ty::Error
                                }
                            }
                        }
                    };
                }
                ty
            }
        }
    }

    pub(crate) fn resolve_visible_type(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Named(name) => self
                .scopes
                .lookup(name)
                .filter(|symbol| matches!(symbol.kind, SymbolKind::Type))
                .map(|symbol| symbol.ty.clone())
                .unwrap_or_else(|| ty.clone()),
            _ => ty.clone(),
        }
    }

    pub(crate) fn designator_is_mutable_target(&self, designator: &Designator) -> bool {
        match designator.parts.first() {
            Some(DesignatorPart::Ident(base, _)) => {
                let Some(symbol) = self.scopes.lookup(base) else {
                    return false;
                };
                symbol.mutable && matches!(symbol.kind, SymbolKind::Var | SymbolKind::Param)
            }
            _ => false,
        }
    }
}
