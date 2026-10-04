//! Qualified enum values and positional payload construction.
//!
//! **Documentation:** `docs/pascal/language/types/enums.md`.

use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{DesignatorPart, Expr};

use crate::check::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;

impl Checker {
    /// Infer enum arguments from payloads and an expected nominal type.
    pub(in crate::check) fn try_check_enum_construction(
        &mut self,
        expression: &Expr,
        expected: Option<&Ty>,
    ) -> Option<Ty> {
        let (designator, arguments, is_call) = match expression {
            Expr::Designator(designator) => (designator, &[][..], false),
            Expr::Call {
                designator, args, ..
            } => (designator, args.as_slice(), true),
            _ => return None,
        };
        if !designator
            .parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(..)))
        {
            return None;
        }
        let source_name = Self::designator_name(designator);
        let name = self.qualified_import_name(&source_name);
        let symbol = self.scopes.lookup(&name)?.clone();
        if !matches!(
            symbol.kind,
            SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor
        ) {
            return None;
        }
        self.resolve_source_name(&source_name, expression.span());
        let Ty::Enum(template) = &symbol.ty else {
            return None;
        };
        let variant_name = source_name.rsplit('.').next()?;
        let variant = template
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))?;
        let span = expression.span();
        if is_call && variant.fields.is_empty() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Payload-less variant `{source_name}` is a value, not a call"),
                format!("Use `{source_name}` without parentheses."),
                span,
            );
        } else if arguments.len() != variant.fields.len() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{source_name}` expects {} argument(s), got {}",
                    variant.fields.len(),
                    arguments.len()
                ),
                format!(
                    "Provide positional values for: {}",
                    variant
                        .fields
                        .iter()
                        .map(|(name, _)| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                span,
            );
        }
        let mut inferred = self.construction_bindings(&symbol.ty, expected);
        let contextual = inferred.clone();
        let mut actual_types = Vec::new();
        for (argument, (_, declared)) in arguments.iter().zip(&variant.fields) {
            let payload_context =
                self.construction_payload_context(&symbol.ty, declared, &contextual);
            actual_types.push(self.check_expr_with_expected(argument, &payload_context));
        }
        for ((_, declared), actual) in variant.fields.iter().zip(&actual_types) {
            // Context supplies final arguments; ordinary field conversions are checked below.
            let declared = declared.substitute(&contextual);
            self.collect_type_param_bindings(
                &declared,
                actual,
                &mut inferred,
                symbol.ty.type_parameters(),
                span,
            );
        }
        self.check_args_only(&arguments[variant.fields.len().min(arguments.len())..]);
        let constructed = self.finish_data_construction(&symbol.ty, &inferred, &source_name, span);
        if let Ty::Enum(enumeration) = &constructed
            && let Some(variant) = enumeration
                .variants
                .iter()
                .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))
        {
            for ((_, declared), actual) in variant.fields.iter().zip(&actual_types) {
                self.check_type_compat(
                    declared,
                    actual,
                    &format!("`{source_name}` argument"),
                    span,
                );
            }
        }
        let key = Self::expr_lookup_key(expression);
        self.expr_types.insert(key, constructed.clone());
        self.propagate_task_bound_expr(expression, key);
        if arguments
            .iter()
            .any(|argument| self.expr_is_task_bound(Self::expr_lookup_key(argument)))
        {
            self.mark_expr_task_bound(key);
        }
        Some(constructed)
    }
}
