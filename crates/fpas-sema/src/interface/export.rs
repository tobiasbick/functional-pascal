//! Extraction and qualification of persistent interfaces from analyzed units.

use fpas_parser::{Decl, Unit, Visibility};
use fpas_unit::interface as artifact;

use crate::check;
use crate::scope::canonical_symbol_name;

use super::conversion::{
    InterfaceConversionError, ty_to_interface_reference, ty_to_interface_type,
};

mod contracts;
mod param_escapes;
mod record_constants;
mod record_defaults;

impl check::Checker {
    /// Extract the canonical public interface of an analyzed source unit.
    pub(super) fn extract_unit_interface(
        &self,
        unit: &Unit,
    ) -> Result<artifact::UnitInterface, InterfaceConversionError> {
        let unit_name = unit.name.parts.join(".");
        let own_types: std::collections::HashSet<String> = unit
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Decl::TypeDef(definition) => Some(canonical_symbol_name(&definition.name)),
                _ => None,
            })
            .collect();
        let mut symbols = Vec::new();
        for declaration in &unit.declarations {
            if declaration.visibility() == Visibility::Private {
                continue;
            }
            let name = declaration_name(declaration);
            let symbol = self.scopes.lookup_root(name).ok_or_else(|| {
                InterfaceConversionError::new(format!(
                    "exported declaration `{name}` has no resolved root symbol"
                ))
            })?;
            let mut ty = if matches!(declaration, Decl::TypeDef(_)) {
                ty_to_interface_type(&symbol.ty)?
            } else {
                ty_to_interface_reference(&symbol.ty)?
            };
            self.apply_record_defaults(declaration, &mut ty)?;
            qualify_owned_type(&mut ty, &unit_name, &own_types);
            let mut kind = exported_symbol_kind(declaration, symbol);
            if let artifact::SymbolKind::Constant(Some(artifact::ConstantValue::EnumValue {
                enum_name,
                ..
            })) = &mut kind
            {
                *enum_name = qualify_owned_name(enum_name, &unit_name, &own_types);
            }
            symbols.push(artifact::InterfaceSymbol {
                constant_record: symbol
                    .constant
                    .as_ref()
                    .filter(|info| info.compile_time)
                    .and_then(|info| info.record.as_ref())
                    .map(|record| record_constants::qualify(record, &unit_name, &own_types)),
                name: name.to_string(),
                qualified_name: format!("{unit_name}.{name}"),
                ty,
                kind,
            });
        }
        Ok(artifact::UnitInterface {
            routine_initialization: self
                .initialization
                .exported
                .iter()
                .map(|(name, summary)| {
                    let mut summary = summary.clone();
                    summary.qualified_name = format!("{unit_name}.{name}");
                    for global in &mut summary.written_globals {
                        if !global.contains('.') {
                            *global = format!("{unit_name}.{global}").to_ascii_lowercase();
                        }
                    }
                    summary
                })
                .collect(),
            routine_contracts: contracts::export_contracts(unit),
            routine_param_escapes: param_escapes::export_param_escapes(
                unit,
                &self.routine_param_escapes,
            ),
            unit_name,
            symbols,
        }
        .canonicalized())
    }
}

/// Return the declared source name of a top-level declaration.
pub(super) fn declaration_name(declaration: &Decl) -> &str {
    match declaration {
        Decl::Const(value) => &value.name,
        Decl::Var(value) => &value.name,
        Decl::TypeDef(value) => &value.name,
        Decl::Function(value) => &value.name,
        Decl::Procedure(value) => &value.name,
    }
}

fn exported_symbol_kind(declaration: &Decl, symbol: &crate::scope::Symbol) -> artifact::SymbolKind {
    match declaration {
        Decl::Const(_) => match &symbol.constant {
            Some(info) if info.compile_time => artifact::SymbolKind::Constant(info.value.clone()),
            _ => artifact::SymbolKind::ComputedConstant,
        },
        Decl::Var(_) => artifact::SymbolKind::Variable,
        Decl::Function(_) => artifact::SymbolKind::Function,
        Decl::Procedure(_) => artifact::SymbolKind::Procedure,
        Decl::TypeDef(_) => artifact::SymbolKind::Type,
    }
}

fn qualify_owned_type(
    ty: &mut artifact::InterfaceType,
    unit_name: &str,
    own_types: &std::collections::HashSet<String>,
) {
    use artifact::InterfaceType::{
        Array, Channel, Dictionary, Enum, Function, GenericParameter, Named, Option, Procedure,
        Record, Result, Task,
    };
    match ty {
        artifact::InterfaceType::Subrange { name, .. } => {
            *name = qualify_owned_name(name, unit_name, own_types);
        }
        Array(inner) | Channel(inner) | Option(inner) | Task(inner) => {
            qualify_owned_type(inner, unit_name, own_types);
        }
        Dictionary(left, right) | Result(left, right) => {
            qualify_owned_type(left, unit_name, own_types);
            qualify_owned_type(right, unit_name, own_types);
        }
        Function(callable) | Procedure(callable) => {
            qualify_callable(callable, unit_name, own_types);
        }
        Record(record) => {
            for argument in &mut record.type_arguments {
                qualify_owned_type(argument, unit_name, own_types);
            }
            // Transparent aliases retain their declaration owner, including member visibility.
            // Documentation: docs/pascal/language/types/type-aliases.md
            let owned = own_types.contains(&canonical_symbol_name(&record.name));
            record.name = qualify_owned_name(&record.name, unit_name, own_types);
            if owned {
                record.owner_unit = Some(unit_name.to_string());
            }
            for field in &mut record.fields {
                qualify_owned_type(&mut field.ty, unit_name, own_types);
                if let Some(artifact::FieldDefaultValue::Scalar(
                    artifact::ConstantValue::EnumValue { enum_name, .. },
                )) = &mut field.default_value
                {
                    *enum_name = qualify_owned_name(enum_name, unit_name, own_types);
                }
            }
            for method in record
                .methods
                .iter_mut()
                .chain(record.static_routines.iter_mut())
            {
                qualify_callable(&mut method.callable, unit_name, own_types);
            }
        }
        Enum(enum_ty) => {
            enum_ty.name = qualify_owned_name(&enum_ty.name, unit_name, own_types);
            for argument in &mut enum_ty.type_arguments {
                qualify_owned_type(argument, unit_name, own_types);
            }
            for variant in &mut enum_ty.variants {
                for field in &mut variant.fields {
                    qualify_owned_type(&mut field.ty, unit_name, own_types);
                }
            }
        }
        Named(name) => *name = qualify_owned_name(name, unit_name, own_types),
        artifact::InterfaceType::EnumApplication { name, arguments } => {
            *name = qualify_owned_name(name, unit_name, own_types);
            for argument in arguments {
                qualify_owned_type(argument, unit_name, own_types);
            }
        }
        artifact::InterfaceType::Application {
            name,
            owner_unit,
            arguments,
        } => {
            if own_types.contains(&canonical_symbol_name(name)) {
                *owner_unit = Some(unit_name.to_owned());
            }
            *name = qualify_owned_name(name, unit_name, own_types);
            for argument in arguments {
                qualify_owned_type(argument, unit_name, own_types);
            }
        }
        GenericParameter(_, _) => {}
        _ => {}
    }
}

fn qualify_callable(
    callable: &mut artifact::CallableType,
    unit_name: &str,
    own_types: &std::collections::HashSet<String>,
) {
    for parameter in &mut callable.parameters {
        qualify_owned_type(&mut parameter.ty, unit_name, own_types);
    }
    if let Some(result) = &mut callable.result {
        qualify_owned_type(result, unit_name, own_types);
    }
}

fn qualify_owned_name(
    name: &str,
    unit_name: &str,
    own_types: &std::collections::HashSet<String>,
) -> String {
    if own_types.contains(&canonical_symbol_name(name)) {
        format!("{unit_name}.{name}")
    } else {
        name.to_string()
    }
}
