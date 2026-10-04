//! Record defaults follow declaration identity through persisted type aliases.
//!
//! **Documentation:** `docs/pascal/language/types/records.md` and
//! `docs/pascal/language/types/type-aliases.md`.

use fpas_parser::Expr;
use fpas_unit::interface as artifact;
use std::sync::Arc;

use crate::check::Checker;

use super::constants::to_interface;
use super::conversion::InterfaceConversionError;

impl Checker {
    /// Restore defaults by nominal identity, including records embedded in collection aliases.
    pub(super) fn export_record_defaults(
        &self,
        ty: &mut artifact::InterfaceType,
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
                        .and_then(|(_, default)| default.as_ref())
                        .map(|default| match default {
                            crate::RecordDefault::Initializer(name) => {
                                artifact::FieldDefault::Initializer {
                                    name: name.clone(),
                                    pure_parameters: self
                                        .default_type_requirements(&record.name, &field.name),
                                }
                            }
                            crate::RecordDefault::Expression(expression) => self
                                .evaluate_static_expression(expression)
                                .and_then(to_interface)
                                .map(artifact::FieldDefault::Constant)
                                .unwrap_or_else(|| artifact::FieldDefault::Initializer {
                                    name: artifact::record_default_initializer(
                                        &record.name,
                                        record.owner_unit.as_deref(),
                                        &field.name,
                                    ),
                                    pure_parameters: self
                                        .default_type_requirements(&record.name, &field.name),
                                }),
                        });
                }
            }
            Type::Array(inner) | Type::Channel(inner) | Type::Option(inner) | Type::Task(inner) => {
                self.export_record_defaults(inner)?;
            }
            Type::Dictionary(left, right) | Type::Result(left, right) => {
                self.export_record_defaults(left)?;
                self.export_record_defaults(right)?;
            }
            // Other nominal components are persisted as qualified references.
            _ => {}
        }
        Ok(())
    }

    /// Install scalar defaults or internal calls with supporting record definitions.
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
                            field.default_value.as_ref().map(|value| match value {
                                artifact::FieldDefault::Constant(value) => {
                                    crate::RecordDefault::Expression(Arc::new(
                                        constant_value_to_expr(value),
                                    ))
                                }
                                artifact::FieldDefault::Initializer {
                                    name,
                                    pure_parameters,
                                } => {
                                    self.install_default_type_requirements(
                                        &record.name,
                                        &field.name,
                                        pure_parameters,
                                    );
                                    crate::RecordDefault::Initializer(name.clone())
                                }
                            }),
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
