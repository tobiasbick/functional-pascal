//! Resolution and classification of value and type designators.
//! See `docs/pascal/language/types/subranges.md` and
//! `docs/pascal/language/types/distinct-types.md`.

use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Recognizes a type name, including parentheses, aliases and unit qualification.
    pub(super) fn is_type_designator(&self, expression: &Expr) -> bool {
        match expression {
            Expr::Paren(inner, _) => self.is_type_designator(inner),
            Expr::Designator(designator) => self.designator_is_type(designator),
            _ => false,
        }
    }

    fn designator_is_type(&self, designator: &Designator) -> bool {
        designator
            .parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(..)))
            && self
                .scopes
                .lookup(&Self::resolve_designator_name(designator))
                .is_some_and(|symbol| symbol.kind == SymbolKind::Type)
    }

    /// Resolves a value name, rejecting type declarations and unspecialized polymorphic intrinsics.
    pub(super) fn check_value_designator_expr(&mut self, designator: &Designator) -> Ty {
        let ty = self.check_designator_expr(designator);
        let name = Self::resolve_designator_name(designator);
        if self
            .scopes
            .lookup(&name)
            .is_some_and(|symbol| symbol.kind == SymbolKind::BuiltinStd)
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Polymorphic operation `{name}` must be called directly"),
                "Declare a wrapper with explicit parameter and result types, and call this operation through its import alias inside the wrapper.",
                designator.span,
            );
            return Ty::Error;
        }
        if !ty.is_error() && self.designator_is_type(designator) {
            let name = Self::resolve_designator_name(designator);
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("The type name `{name}` cannot be used as a value"),
                format!("Use a value expression or a conversion/construction call, such as `{name}(Value)` for a scalar type."),
                designator.span,
            );
            Ty::Error
        } else {
            ty
        }
    }

    /// Checks an already-classified type operand while retaining expression type metadata.
    pub(super) fn check_type_operand(&mut self, expression: &Expr) -> Ty {
        let ty = match expression {
            Expr::Paren(inner, _) => self.check_type_operand(inner),
            Expr::Designator(designator) => self.check_designator_expr(designator),
            _ => Ty::Error,
        };
        self.expr_types
            .insert(Self::expr_lookup_key(expression), ty.clone());
        ty
    }

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
        self.record_initialization_reference(designator);
        self.record_contract_read(designator, parts);
        let only_ident_chain = parts
            .iter()
            .all(|p| matches!(p, DesignatorPart::Ident(_, _)));

        if only_ident_chain {
            let full_name = Self::resolve_designator_parts_name(parts);
            self.ensure_fq_std_unit_loaded(&full_name);
            if let Some(symbol) = self.scopes.lookup(&full_name) {
                if parts.len() == designator.parts.len() && symbol.kind == SymbolKind::EnumMember {
                    self.enum_members
                        .insert(crate::designator_lookup_key(designator));
                }
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

                    let repair_name = if is_qualified_ident_chain
                        && self.scopes.imports.hidden_path_hint(&full_name).is_none()
                    {
                        full_name.as_str()
                    } else {
                        first.as_str()
                    };
                    if let Some(qualification_hint) = self.qualification_hint(repair_name) {
                        let message =
                            format!("Name `{repair_name}` requires an explicit qualifier");
                        self.error_with_code(
                            fpas_diagnostics::codes::SEMA_UNQUALIFIED_NAME,
                            message,
                            qualification_hint,
                            designator.span,
                        );
                        return Ty::Error;
                    }

                    if is_qualified_ident_chain
                        && let Some(unit) =
                            crate::std_units::missing_std_unit(&full_name, &self.loaded_std_units)
                    {
                        self.error_with_code(
                            SEMA_UNKNOWN_NAME,
                            format!("Unit `{unit}` is not imported"),
                            crate::std_units::missing_import_hint(&unit),
                            designator.span,
                        );
                        return Ty::Error;
                    }

                    let hint = if let Some(hint) = self.scopes.imports.hidden_path_hint(&full_name)
                    {
                        hint
                    } else if is_qualified_ident_chain {
                        if crate::std_units::looks_like_std_qualified_name(&full_name) {
                            self.hint_unknown_callable(&full_name, &[])
                        } else {
                            "Check that the unit is listed in `uses` and that the symbol is public. Private unit members are not visible outside their unit.".to_string()
                        }
                    } else if crate::std_units::looks_like_std_qualified_name(first) {
                        self.hint_unknown_callable(first, &[])
                    } else if let Some(hint) = self.foreign_builtin_hint(first, &[]) {
                        hint
                    } else {
                        "Check spelling or declare the variable or constant.".to_string()
                    };

                    let message = if is_qualified_ident_chain {
                        format!("Undefined identifier `{full_name}`")
                    } else {
                        format!("Undefined identifier `{first}`")
                    };

                    self.error_with_code(SEMA_UNKNOWN_NAME, message, hint, designator.span);
                    if !is_qualified_ident_chain {
                        let fix = self.foreign_builtin_fix(first, &[], designator.span);
                        self.attach_fix_to_last_error(fix);
                    }
                    return Ty::Error;
                };

                let designator_key = crate::designator_lookup_key(designator);
                let trailing = parts.len().saturating_sub(base_part_count);
                for (offset, part) in parts[base_part_count..].iter().enumerate() {
                    ty = self.resolve_visible_type(&ty);

                    ty = match part {
                        DesignatorPart::Ident(field, span) => {
                            let is_last = offset + 1 == trailing;
                            let bound_key = if is_last {
                                Some((designator_key, parts.len() - 1))
                            } else {
                                None
                            };
                            self.check_record_member_access(&ty, field, *span, bound_key)
                        }
                        DesignatorPart::Index(index_expr, span) => {
                            self.check_index_access(&ty, index_expr, *span)
                        }
                    };
                    let span = match part {
                        DesignatorPart::Ident(_, span) | DesignatorPart::Index(_, span) => span,
                    };
                    self.path_types
                        .insert((span.source_id, span.offset), ty.clone());
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
        self.designator_root_symbol(parts)
            .map(|(symbol, count)| (symbol.ty.clone(), count))
    }

    /// Resolves the longest visible binding prefix, including qualified unit exports.
    pub(crate) fn designator_root_symbol(
        &self,
        parts: &[DesignatorPart],
    ) -> Option<(&crate::scope::Symbol, usize)> {
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
                resolved = Some((symbol, index + 1));
            }
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
                if !Ty::Integer.assignment_compatible_with(&index_ty) {
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
                if !key_ty.assignment_compatible_with(&index_ty) {
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
                if !Ty::Integer.assignment_compatible_with(&index_ty) {
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
        self.designator_root_symbol(&designator.parts)
            .is_some_and(|(symbol, _)| {
                symbol.mutable && matches!(symbol.kind, SymbolKind::Var | SymbolKind::Param)
            })
    }
}
