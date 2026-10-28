//! Installation of compiled-unit interface symbols into semantic scopes.
//! See `docs/pascal/program-structure/units.md` and `docs/pascal/language/types/generics.md`.

use fpas_parser::Expr;
use fpas_unit::interface as artifact;

use crate::check;
use crate::scope::{Symbol, SymbolKind as SemaSymbolKind, canonical_symbol_name};

use super::conversion::{InterfaceConversionError, interface_symbol_to_sema, interface_type_to_ty};

impl check::Checker {
    /// Install only qualified type definitions from transitive supporting interfaces.
    pub(crate) fn install_supporting_interface_types(
        &mut self,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        for interface in interfaces {
            self.needs_error_constructors |= !interface.errors.is_empty();
            interface
                .errors
                .validate()
                .map_err(InterfaceConversionError::new)?;
            for schema in &interface.errors.schemas {
                self.imported_error_schemas
                    .insert(schema.declaration.clone(), schema.clone());
            }
            for summary in &interface.routine_initialization {
                self.initialization
                    .imported
                    .insert(summary.qualified_name.to_ascii_lowercase(), summary.clone());
            }
            for routine in &interface.routine_param_escapes {
                self.scopes.set_param_escapes(
                    &routine.qualified_name,
                    std::sync::Arc::from(routine.escapes.as_slice()),
                );
            }
            for exported in &interface.symbols {
                self.install_native_error_dependencies(&exported.ty);
                if exported.kind != artifact::SymbolKind::Type {
                    continue;
                }
                self.install_interface_type_metadata(exported, interface_symbol_to_sema(exported)?);
            }
        }
        Ok(())
    }

    /// Install only canonical public symbols and alias-qualified repair targets.
    pub(crate) fn install_interfaces(
        &mut self,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        for interface in interfaces {
            self.needs_error_constructors |= !interface.errors.is_empty();
            interface
                .errors
                .validate()
                .map_err(InterfaceConversionError::new)?;
            for schema in &interface.errors.schemas {
                self.imported_error_schemas
                    .insert(schema.declaration.clone(), schema.clone());
            }
            for summary in &interface.routine_initialization {
                self.initialization
                    .imported
                    .insert(summary.qualified_name.to_ascii_lowercase(), summary.clone());
            }
            for routine in &interface.routine_param_escapes {
                self.scopes.set_param_escapes(
                    &routine.qualified_name,
                    std::sync::Arc::from(routine.escapes.as_slice()),
                );
            }
            for exported in &interface.symbols {
                self.install_native_error_dependencies(&exported.ty);
                let mut symbol = interface_symbol_to_sema(exported)?;
                if !self.type_can_contain_callable(&symbol.ty) {
                    symbol.capability = crate::TransferCapability::Transferable;
                }
                self.scopes.imports.define_source(
                    &interface.unit_name,
                    &exported.qualified_name,
                    symbol.clone(),
                );
                if exported.kind == artifact::SymbolKind::Type {
                    self.install_interface_type_metadata(exported, symbol);
                }
                self.install_imported_record_defaults(exported);
                self.imported_candidates
                    .entry(canonical_symbol_name(&exported.name))
                    .or_default()
                    .extend(
                        self.scopes
                            .imports
                            .source_name_for(&interface.unit_name, &exported.qualified_name),
                    );
                self.install_imported_enum_variants(&interface.unit_name, exported)?;
            }
        }

        Ok(())
    }

    // Nominal references retain the original identity even when only an alias is imported.
    fn install_interface_type_metadata(
        &mut self,
        exported: &artifact::InterfaceSymbol,
        symbol: Symbol,
    ) {
        self.scopes
            .imports
            .define_metadata(&exported.qualified_name, symbol.clone());
        // A concrete generic alias is not a definition of its uninstantiated base type.
        let original_name = match &exported.ty {
            artifact::InterfaceType::Enum(enumeration) if enumeration.type_arguments.is_empty() => {
                Some(&enumeration.name)
            }
            artifact::InterfaceType::Record(record) if record.type_arguments.is_empty() => {
                Some(&record.name)
            }
            _ => None,
        };
        if let Some(name) = original_name {
            self.scopes.imports.define_metadata(name, symbol);
        }
    }

    fn install_imported_record_defaults(&mut self, exported: &artifact::InterfaceSymbol) {
        let artifact::InterfaceType::Record(record) = &exported.ty else {
            return;
        };
        // Transparent aliases share one retained allocation for each original default.
        if self.record_defaults.contains_key(&record.name) {
            return;
        }
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
                    field.default_value.as_ref().map(|value| {
                        let expression = std::sync::Arc::new(field_default_to_expr(value));
                        let key = Self::expr_lookup_key(&expression);
                        self.record_default_constants.insert(key, true);
                        self.record_default_values.insert(
                            key,
                            match value {
                                artifact::FieldDefaultValue::Scalar(value) => {
                                    Some(artifact::RecordConstantField::Scalar(value.clone()))
                                }
                                artifact::FieldDefaultValue::OptionNone => None,
                            },
                        );
                        expression
                    }),
                )
            })
            .collect();
        self.record_defaults.insert(record.name.clone(), fields);
    }

    fn install_imported_enum_variants(
        &mut self,
        unit: &str,
        exported: &artifact::InterfaceSymbol,
    ) -> Result<(), InterfaceConversionError> {
        if exported.kind != artifact::SymbolKind::Type {
            return Ok(());
        }
        let artifact::InterfaceType::Enum(enum_ty) = &exported.ty else {
            return Ok(());
        };
        let enum_symbol_ty = interface_type_to_ty(&exported.ty)?;
        for variant in &enum_ty.variants {
            let kind = if variant.fields.is_empty() {
                SemaSymbolKind::EnumMember
            } else {
                SemaSymbolKind::EnumVariantConstructor
            };
            let symbol = Symbol {
                constant: None,
                ty: enum_symbol_ty.clone(),
                mutable: false,
                kind,
                capability: crate::TransferCapability::Transferable,
            };
            let fully_qualified = format!("{}.{}", exported.qualified_name, variant.name);
            self.scopes
                .imports
                .define_source(unit, &fully_qualified, symbol);
            for key in [
                variant.name.clone(),
                format!("{}.{}", exported.name, variant.name),
            ] {
                self.imported_candidates
                    .entry(canonical_symbol_name(&key))
                    .or_default()
                    .extend(self.scopes.imports.source_name_for(unit, &fully_qualified));
            }
        }
        Ok(())
    }
}

fn field_default_to_expr(value: &artifact::FieldDefaultValue) -> Expr {
    let span = fpas_lexer::Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
        source_id: 0,
    };
    match value {
        artifact::FieldDefaultValue::OptionNone => Expr::OptionNone(span),
        artifact::FieldDefaultValue::Scalar(value) => match value {
            artifact::ConstantValue::Integer(value) => Expr::Integer(*value, span),
            artifact::ConstantValue::Real(bits) => Expr::Real(f64::from_bits(*bits), span),
            artifact::ConstantValue::Boolean(value) => Expr::Bool(*value, span),
            artifact::ConstantValue::String(value) => Expr::Str(value.clone(), span),
            artifact::ConstantValue::EnumValue { backing_value, .. } => {
                // The cached interface value retains enum identity; lowering needs its scalar.
                Expr::Integer(*backing_value, span)
            }
        },
    }
}
