use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::{Designator, DesignatorPart};

mod roots;

impl Checker {
    pub(crate) fn check_designator_expr(&mut self, designator: &Designator) -> Ty {
        self.check_designator_prefix_expr(designator, designator.parts.len())
    }

    /// Type-check a designator that stands alone as a value expression.
    ///
    /// Type names may prefix variant constructors, but are not values themselves.
    pub(crate) fn check_designator_value_expr(&mut self, designator: &Designator) -> Ty {
        if designator
            .parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(_, _)))
        {
            let raw_name = Self::resolve_designator_parts_name(&designator.parts);
            let full_name = self.qualified_import_name(&raw_name);
            if self
                .scopes
                .lookup(&full_name)
                .is_some_and(|symbol| matches!(symbol.kind, SymbolKind::Type))
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Type `{raw_name}` is not a value"),
                    format!(
                        "Use a value of type `{raw_name}`, for example a variable or the constructor `{raw_name}(Field := Value)`."
                    ),
                    designator.span,
                );
                return Ty::Error;
            }
        }
        self.check_designator_expr(designator)
    }

    /// Type-check a leading portion of a designator without cloning its index expressions.
    /// Imported storage roots retain the same alias-only policy before index projection.
    pub(crate) fn check_designator_prefix_expr(
        &mut self,
        designator: &Designator,
        part_count: usize,
    ) -> Ty {
        let parts = &designator.parts[..part_count.min(designator.parts.len())];
        self.check_pure_designator(parts, designator.span);
        let only_ident_chain = parts
            .iter()
            .all(|p| matches!(p, DesignatorPart::Ident(_, _)));

        if only_ident_chain {
            let raw_name = Self::resolve_designator_parts_name(parts);
            let full_name = self.resolve_source_name(&raw_name, designator.span);
            self.ensure_fq_std_unit_loaded(&full_name);
            if let Some(symbol) = self.scopes.lookup(&full_name) {
                return symbol.ty.clone();
            }
        } else if let Some((_, consumed)) = self.designator_root_symbol(parts) {
            let raw_root = Self::resolve_designator_parts_name(&parts[..consumed]);
            self.resolve_source_name(&raw_root, designator.span);
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
                let resolved_base = self.designator_root_symbol(parts);
                let Some((root, base_part_count)) = resolved_base else {
                    let raw_name = Self::resolve_designator_parts_name(parts);
                    let full_name = self.resolve_source_name(&raw_name, designator.span);
                    let is_qualified_ident_chain = parts.len() > 1
                        && parts
                            .iter()
                            .all(|part| matches!(part, DesignatorPart::Ident(_, _)));

                    if is_qualified_ident_chain
                        && let Some(unit) =
                            crate::std_units::missing_std_unit(&full_name, &self.loaded_std_units)
                    {
                        self.error_with_code(
                            SEMA_UNKNOWN_NAME,
                            format!("Unit `{unit}` is not imported"),
                            format!(
                                "Add `{unit}` to the `uses` clause; qualified names do not import a unit."
                            ),
                            designator.span,
                        );
                        return Ty::Error;
                    }

                    let hint = if let Some(hint) = self.import_name_hint(first) {
                        hint
                    } else if is_qualified_ident_chain {
                        if crate::std_units::looks_like_std_qualified_name(&full_name) {
                            self.hint_unknown_callable(&full_name)
                        } else {
                            "Check that the unit is listed in `uses` and that the symbol is public. Private unit members are not visible outside their unit.".to_string()
                        }
                    } else if crate::std_units::looks_like_std_qualified_name(first) {
                        self.hint_unknown_callable(first)
                    } else {
                        self.import_name_hint(first).unwrap_or_else(|| {
                            "Check spelling or declare the variable or constant.".to_string()
                        })
                    };

                    let message = if is_qualified_ident_chain {
                        format!("Undefined identifier `{full_name}`")
                    } else {
                        format!("Undefined identifier `{first}`")
                    };

                    self.error_with_code(SEMA_UNKNOWN_NAME, message, hint, designator.span);
                    return Ty::Error;
                };

                let mut ty = root.ty.clone();
                for part in &parts[base_part_count..] {
                    ty = self.resolve_visible_type(&ty);

                    ty = match part {
                        DesignatorPart::Ident(field, span) => {
                            self.check_record_field_access(&ty, field, *span)
                        }
                        DesignatorPart::Index(index_expr, span) => {
                            self.check_index_access(&ty, index_expr, *span)
                        }
                    };
                    self.projection_types.insert(
                        crate::designator_part_lookup_key(part),
                        self.resolve_visible_type(&ty),
                    );
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

    pub(crate) fn resolve_visible_type(&self, ty: &Ty) -> Ty {
        let mut resolved = ty.clone();
        let mut visited = std::collections::HashSet::new();
        while let Ty::Named(name) | Ty::Applied(name, _) = &resolved {
            let key = crate::scope::canonical_symbol_name(name);
            if !visited.insert(key) {
                break;
            }
            let Some(definition) = self
                .scopes
                .lookup(name)
                .filter(|symbol| matches!(symbol.kind, SymbolKind::Type))
                .map(|symbol| symbol.ty.clone())
                .or_else(|| crate::std_registry::intrinsic_type(name))
            else {
                break;
            };
            resolved = if let Ty::Applied(_, arguments) = &resolved {
                definition.instantiate(arguments).unwrap_or(Ty::Error)
            } else {
                definition
            };
        }
        resolved
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

    /// Reject writes through a string index in an already type-checked target.
    ///
    /// Projection types preserve the receiver at each step without checking index
    /// expressions again. String indices read character values, not writable storage.
    /// **Documentation:** `docs/pascal/language/basics/operators.md#string-indexing`
    pub(crate) fn reject_string_index_assignment(&mut self, designator: &Designator) -> bool {
        let Some((root, base_part_count)) = self.designator_root_symbol(&designator.parts) else {
            return false;
        };
        let mut ty = root.ty.clone();
        for part in &designator.parts[base_part_count..] {
            if matches!(self.resolve_visible_type(&ty), Ty::String)
                && let DesignatorPart::Index(_, span) = part
            {
                self.error_with_code(
                    SEMA_IMMUTABLE_ASSIGNMENT,
                    "String indices are read-only",
                    "Read characters with Text[0]. To change text, assign a whole string to a mutable binding, field, or collection element.",
                    *span,
                );
                return true;
            }
            let Some(projected) = self
                .projection_types
                .get(&crate::designator_part_lookup_key(part))
            else {
                return false;
            };
            ty = projected.clone();
        }
        false
    }
}
