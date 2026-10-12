//! Declaration-evaluated record defaults in persistent unit interfaces.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use std::collections::HashSet;

use fpas_diagnostics::codes::SEMA_NON_CONSTANT_EXPRESSION;
use fpas_parser::{Decl, Expr, Unit, Visibility};
use fpas_unit::interface::{FieldDefaultValue, InterfaceType, RecordConstantField};

use crate::check::Checker;
use crate::types::Ty;

use super::InterfaceConversionError;

impl Checker {
    /// Diagnose unsupported exported defaults at their original declaration expression.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md`
    pub(crate) fn validate_exported_record_defaults(&mut self, unit: &Unit) {
        let mut seen = HashSet::new();
        for declaration in &unit.declarations {
            let Decl::TypeDef(definition) = declaration else {
                continue;
            };
            if definition.visibility != Visibility::Public {
                continue;
            }
            let Some(symbol) = self.scopes.lookup_root(&definition.name) else {
                continue;
            };
            let Ty::Record(record) = &symbol.ty else {
                continue;
            };
            let Some(defaults) = self.record_defaults.get(&record.name).cloned() else {
                continue;
            };
            for (field, expression) in defaults {
                let Some(expression) = expression else {
                    continue;
                };
                if !seen.insert(Self::expr_lookup_key(&expression)) {
                    continue;
                }
                if self.exported_field_default(&expression).is_none() {
                    self.error_with_code(
                        SEMA_NON_CONSTANT_EXPRESSION,
                        format!("Exported record field `{field}` requires a valid scalar compile-time default or None"),
                        "Use a scalar constant that evaluates successfully, for example `Value: integer := 1 + 2;`, or `None` for an optional field. Supply computed or aggregate values when constructing the record.",
                        expression.span(),
                    );
                }
            }
        }
    }

    /// Preserve checked defaults when exporting a concrete record or transparent alias.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md`
    pub(super) fn apply_record_defaults(
        &self,
        declaration: &Decl,
        ty: &mut InterfaceType,
    ) -> Result<(), InterfaceConversionError> {
        if !matches!(declaration, Decl::TypeDef(_)) {
            return Ok(());
        }
        if let InterfaceType::Record(record) = ty
            && let Some(defaults) = self.record_defaults.get(&record.name)
        {
            for field in &mut record.fields {
                field.default_value = defaults
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                    .and_then(|(_, value)| value.as_deref())
                    .map(|expression| {
                        self.exported_field_default(expression).ok_or_else(|| {
                            InterfaceConversionError::new(
                                "exported record default has no checked interface value",
                            )
                        })
                    })
                    .transpose()?;
            }
        }
        Ok(())
    }

    fn exported_field_default(&self, expression: &Expr) -> Option<FieldDefaultValue> {
        let mut inner = expression;
        while let Expr::Paren(value, _) = inner {
            inner = value;
        }
        if matches!(inner, Expr::OptionNone(_)) {
            return Some(FieldDefaultValue::OptionNone);
        }
        match self
            .record_default_values
            .get(&Self::expr_lookup_key(expression))?
        {
            Some(RecordConstantField::Scalar(value)) => {
                Some(FieldDefaultValue::Scalar(value.clone()))
            }
            _ => None,
        }
    }
}
