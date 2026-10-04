//! Record defaults follow declaration identity through persisted type aliases.
//!
//! **Documentation:** `docs/pascal/language/types/records.md` and
//! `docs/pascal/language/types/type-aliases.md`.

use fpas_parser::Expr;
use fpas_unit::interface as artifact;
use std::sync::Arc;

use crate::check::Checker;

use super::constants::ScalarConstants;
use super::conversion::InterfaceConversionError;

impl Checker {
    /// Restore defaults by nominal identity, including records embedded in collection aliases.
    pub(super) fn export_record_defaults(
        &self,
        ty: &mut artifact::InterfaceType,
        constants: &ScalarConstants,
    ) -> Result<(), InterfaceConversionError> {
        use artifact::InterfaceType as Type;
        match ty {
            Type::Record(record) => {
                let Some(defaults) = self.record_defaults.get(&record.name) else {
                    return Ok(());
                };
                for field in &mut record.fields {
                    field.default_value = defaults
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                        .and_then(|(_, default)| default.as_deref())
                        .map(|expression| {
                            constants.value(expression).ok_or_else(|| {
                                InterfaceConversionError::new(
                                    "exported record field defaults must be scalar constant expressions",
                                )
                            })
                        })
                        .transpose()?;
                }
            }
            Type::Array(inner) | Type::Channel(inner) | Type::Option(inner) | Type::Task(inner) => {
                self.export_record_defaults(inner, constants)?;
            }
            Type::Dictionary(left, right) | Type::Result(left, right) => {
                self.export_record_defaults(left, constants)?;
                self.export_record_defaults(right, constants)?;
            }
            // Other nominal components are persisted as qualified references.
            _ => {}
        }
        Ok(())
    }

    /// Install scalar defaults with direct and supporting record definitions.
    pub(super) fn install_imported_record_defaults(&mut self, ty: &artifact::InterfaceType) {
        use artifact::InterfaceType as Type;
        match ty {
            Type::Record(record) => {
                if !record
                    .fields
                    .iter()
                    .any(|field| field.default_value.is_some())
                {
                    return;
                }
                let fields = record
                    .fields
                    .iter()
                    .map(|field| {
                        (
                            field.name.clone(),
                            field
                                .default_value
                                .as_ref()
                                .map(|value| Arc::new(constant_value_to_expr(value))),
                        )
                    })
                    .collect();
                self.record_defaults.insert(record.name.clone(), fields);
            }
            Type::Array(inner) | Type::Channel(inner) | Type::Option(inner) | Type::Task(inner) => {
                self.install_imported_record_defaults(inner);
            }
            Type::Dictionary(left, right) | Type::Result(left, right) => {
                self.install_imported_record_defaults(left);
                self.install_imported_record_defaults(right);
            }
            _ => {}
        }
    }
}

fn constant_value_to_expr(value: &artifact::ConstantValue) -> Expr {
    let span = fpas_lexer::Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
        source_id: 0,
    };
    match value {
        artifact::ConstantValue::Integer(value) => Expr::Integer(*value, span),
        artifact::ConstantValue::Real(bits) => Expr::Real(f64::from_bits(*bits), span),
        artifact::ConstantValue::Boolean(value) => Expr::Bool(*value, span),
        artifact::ConstantValue::String(value) => Expr::Str(value.clone(), span),
        artifact::ConstantValue::EnumValue {
            enum_name,
            variant_name,
            ..
        } => Expr::Designator(fpas_parser::Designator {
            parts: enum_name
                .split('.')
                .chain(std::iter::once(variant_name.as_str()))
                .map(|part| fpas_parser::DesignatorPart::Ident(part.to_string(), span))
                .collect(),
            span,
        }),
    }
}
