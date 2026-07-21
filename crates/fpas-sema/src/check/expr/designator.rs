use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_AMBIGUOUS_IMPORTED_NAME, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};
use fpas_parser::{Designator, DesignatorPart};

impl Checker {
    pub(crate) fn check_designator_expr(&mut self, designator: &Designator) -> Ty {
        self.check_designator_prefix_expr(designator, designator.parts.len())
    }

    /// Type-check a leading portion of a designator without cloning its index expressions.
    pub(crate) fn check_designator_prefix_expr(
        &mut self,
        designator: &Designator,
        part_count: usize,
    ) -> Ty {
        let parts = &designator.parts[..part_count.min(designator.parts.len())];
        let only_ident_chain = parts
            .iter()
            .all(|p| matches!(p, DesignatorPart::Ident(_, _)));

        if only_ident_chain {
            let full_name = Self::resolve_designator_parts_name(parts);
            self.ensure_fq_std_unit_loaded(&full_name);
            if let Some(symbol) = self.scopes.lookup(&full_name) {
                return symbol.ty.clone();
            }
        }
        self.check_designator_path(designator, parts)
    }

    fn check_designator_path(&mut self, designator: &Designator, parts: &[DesignatorPart]) -> Ty {
        if parts.is_empty() {
            return Ty::Error;
        }

        match &parts[0] {
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
                let resolved_base = self.resolve_designator_base(parts);
                let Some((mut ty, base_part_count)) = resolved_base else {
                    let full_name = Self::resolve_designator_parts_name(parts);
                    let is_qualified_ident_chain = parts.len() > 1
                        && parts
                            .iter()
                            .all(|part| matches!(part, DesignatorPart::Ident(_, _)));

                    if let Some(ambiguous_hint) = self.ambiguous_hint(first) {
                        self.error_with_code(
                            SEMA_AMBIGUOUS_IMPORTED_NAME,
                            format!("Ambiguous name `{first}`"),
                            ambiguous_hint,
                            designator.span,
                        );
                        return Ty::Error;
                    }

                    let hint = if is_qualified_ident_chain {
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

                    let message = if is_qualified_ident_chain {
                        format!("Undefined identifier `{full_name}`")
                    } else {
                        format!("Undefined identifier `{first}`")
                    };

                    self.error_with_code(SEMA_UNKNOWN_NAME, message, hint, designator.span);
                    return Ty::Error;
                };

                let designator_key = crate::designator_lookup_key(designator);
                let trailing = parts.len().saturating_sub(base_part_count);
                for (offset, part) in parts[base_part_count..].iter().enumerate() {
                    ty = self.resolve_visible_type(&ty);

                    ty = match part {
                        DesignatorPart::Ident(field, span) => {
                            let is_last = offset + 1 == trailing;
                            let property_key = Some((designator_key, base_part_count + offset));
                            let bound_key = if is_last {
                                Some((designator_key, parts.len() - 1))
                            } else {
                                None
                            };
                            self.check_record_member_access(
                                &ty,
                                field,
                                *span,
                                property_key,
                                bound_key,
                            )
                        }
                        DesignatorPart::Index(index_expr, span) => {
                            self.check_index_access(&ty, index_expr, *span)
                        }
                    };
                }
                ty
            }
        }
    }

    fn resolve_designator_parts_name(parts: &[DesignatorPart]) -> String {
        let mut result = String::new();
        for part in parts {
            if let DesignatorPart::Ident(name, _) = part {
                if !result.is_empty() {
                    result.push('.');
                }
                result.push_str(name);
            }
        }
        result
    }

    fn resolve_designator_base(&self, parts: &[DesignatorPart]) -> Option<(Ty, usize)> {
        let mut qualified = String::new();
        let mut resolved = None;
        for (index, part) in parts.iter().enumerate() {
            let DesignatorPart::Ident(name, _) = part else {
                break;
            };
            if !qualified.is_empty() {
                qualified.push('.');
            }
            qualified.push_str(name);
            if let Some(symbol) = self.scopes.lookup(&qualified)
                && matches!(
                    symbol.kind,
                    SymbolKind::Const | SymbolKind::Var | SymbolKind::Param | SymbolKind::ForVar
                )
            {
                resolved = Some((symbol.ty.clone(), index + 1));
            }
        }
        resolved
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

    /// Resolve `[index]` on a value whose static type is `ty` (aliases already resolved by caller).
    pub(crate) fn check_index_access(
        &mut self,
        ty: &Ty,
        index_expr: &fpas_parser::Expr,
        span: fpas_lexer::Span,
    ) -> Ty {
        let index_ty = self.check_expr(index_expr);

        match ty {
            Ty::Array(inner) => {
                if index_ty != Ty::Integer && !index_ty.is_error() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Array index must be integer",
                        "Use an integer index expression.",
                        index_expr.span(),
                    );
                }
                *inner.clone()
            }
            Ty::Dict(key_ty, val_ty) => {
                if !index_ty.compatible_with(key_ty) && !index_ty.is_error() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Dict key type mismatch: expected `{key_ty}`, got `{index_ty}`"),
                        "Use a key matching the dict's key type.",
                        index_expr.span(),
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
                        index_expr.span(),
                    );
                }
                Ty::String
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "Indexed value is not an array, dict, or string",
                    "Use `A[I]` only on array, dict, or string values.",
                    span,
                );
                Ty::Error
            }
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
