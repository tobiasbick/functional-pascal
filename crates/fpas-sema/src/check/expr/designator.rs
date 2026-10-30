//! Resolution and classification of value and type designators.
//! See `docs/pascal/language/types/subranges.md` and
//! `docs/pascal/language/types/distinct-types.md`.

use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;
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
        self.designator_parts_denote_type(&designator.parts)
    }

    /// Distinguish a type owner from a captured value without cloning index expressions.
    pub(in crate::check) fn designator_parts_denote_type(&self, parts: &[DesignatorPart]) -> bool {
        parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(..)))
            && self
                .scopes
                .lookup(&Self::resolve_designator_parts_name(parts))
                .is_some_and(|symbol| symbol.kind == SymbolKind::Type)
    }

    /// Resolves values and checked native callable references.
    pub(super) fn check_value_designator_expr(
        &mut self,
        designator: &Designator,
        expected: Option<&Ty>,
    ) -> Ty {
        if let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice()
            && let Some(constructor) = crate::types::PayloadConstructor::from_name(name)
        {
            self.enum_members
                .insert(crate::designator_lookup_key(designator));
            return Ty::Function(constructor.signature());
        }
        if let Some(ty) = self.try_native_callable_designator(designator, expected) {
            return ty;
        }
        let ty = self.check_designator_expr(designator);
        let name = Self::resolve_designator_name(designator);
        if self
            .scopes
            .lookup(&name)
            .is_some_and(|symbol| symbol.kind == SymbolKind::BuiltinStd)
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Polymorphic operation `{name}` needs a callable signature"),
                "Use an explicit callable type or call the operation directly.",
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
                let ty = symbol.ty.clone();
                if parts.len() == designator.parts.len()
                    && matches!(
                        symbol.kind,
                        SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor
                    )
                {
                    self.enum_members
                        .insert(crate::designator_lookup_key(designator));
                }
                if let Some(DesignatorPart::Ident(_, span)) = parts.last() {
                    if symbol.kind == SymbolKind::EnumMember {
                        self.enum_member_paths.insert((span.source_id, span.offset));
                    }
                    self.path_types
                        .insert((span.source_id, span.offset), ty.clone());
                }
                return ty;
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
                if let Some(DesignatorPart::Ident(_, span)) =
                    parts.get(base_part_count.saturating_sub(1))
                {
                    self.path_types
                        .insert((span.source_id, span.offset), ty.clone());
                }
                let trailing = parts.len().saturating_sub(base_part_count);
                for (offset, part) in parts[base_part_count..].iter().enumerate() {
                    ty = self.resolve_visible_type(&ty);

                    ty = match part {
                        DesignatorPart::Ident(field, span) => {
                            self.member_receivers
                                .insert((span.source_id, span.offset), ty.clone());
                            let is_last = offset + 1 == trailing;
                            let bound_key = if is_last {
                                Some((designator_key, parts.len() - 1, designator.span))
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
            Ty::ByteBuffer => {
                if !Ty::Integer.assignment_compatible_with(&index_ty) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Byte buffer index must be an integer",
                        "Use a zero-based integer byte index.",
                        index_expr.span(),
                    );
                }
                Ty::Integer
            }
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
            && self.designator_read_only_index(designator).is_none()
    }

    /// Indexed string and byte reads never expose writable scalar storage.
    pub(crate) fn designator_read_only_index(
        &self,
        target: &Designator,
    ) -> Option<(Span, &'static str)> {
        let (symbol, root_parts) = self.designator_root_symbol(&target.parts)?;
        let mut current = symbol.ty.clone();
        for part in &target.parts[root_parts..] {
            current = self.resolve_visible_type(&current);
            current = match (part, &current) {
                (DesignatorPart::Index(_, span), Ty::String | Ty::ByteBuffer) => {
                    return Some((
                        *span,
                        if current == Ty::ByteBuffer {
                            "SetAtInPlace"
                        } else {
                            "SetCharAtInPlace"
                        },
                    ));
                }
                (DesignatorPart::Index(..), Ty::Array(inner) | Ty::Dict(_, inner)) => {
                    (**inner).clone()
                }
                (DesignatorPart::Ident(name, _), Ty::Record(record)) => record
                    .fields
                    .iter()
                    .find(|(field, _)| field.eq_ignore_ascii_case(name))?
                    .1
                    .clone(),
                _ => return None,
            };
        }
        None
    }
}
