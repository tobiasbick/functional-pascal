//! Extraction and qualification of persistent interfaces from analyzed units.

use fpas_parser::{Decl, Unit, Visibility};
use fpas_unit::interface as artifact;

use crate::check;
use crate::scope::canonical_symbol_name;

use super::constants::StaticConstants;
use super::conversion::{
    InterfaceConversionError, ty_to_interface_reference, ty_to_interface_type,
};

impl check::Checker {
    /// Extract the canonical public interface of an analyzed source unit.
    pub(super) fn extract_unit_interface(
        &self,
        unit: &Unit,
        _interfaces: &[artifact::UnitInterface],
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
            self.export_record_defaults(&mut ty)?;
            qualify_owned_type(&mut ty, &unit_name, &own_types);
            symbols.push(artifact::InterfaceSymbol {
                name: name.to_string(),
                qualified_name: format!("{unit_name}.{name}"),
                ty,
                kind: exported_symbol_kind(
                    declaration,
                    &self.static_constants,
                    symbol.kind == crate::scope::SymbolKind::Const,
                ),
            });
        }
        Ok(artifact::UnitInterface { unit_name, symbols }.canonicalized())
    }
}

/// Return the declared source name of a top-level declaration.
pub(super) fn declaration_name(declaration: &Decl) -> &str {
    match declaration {
        Decl::Const(value) | Decl::Var(value) => &value.name,
        Decl::TypeDef(value) => &value.name,
        Decl::Function(value) => &value.name,
        Decl::Procedure(value) => &value.name,
    }
}

fn exported_symbol_kind(
    declaration: &Decl,
    constants: &StaticConstants,
    is_static: bool,
) -> artifact::SymbolKind {
    match declaration {
        Decl::Const(definition) if is_static => constants.binding_interface_kind(definition.span),
        Decl::Const(_) => artifact::SymbolKind::Variable,
        Decl::Var(_) => artifact::SymbolKind::MutableVariable,
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
            // Transparent aliases retain their declaration owner, including field visibility.
            // Documentation: docs/pascal/language/types/type-aliases.md
            let owned = own_types.contains(&canonical_symbol_name(&record.name));
            record.name = qualify_owned_name(&record.name, unit_name, own_types);
            if owned {
                record.owner_unit = Some(unit_name.to_string());
            }
            for field in &mut record.fields {
                qualify_owned_type(&mut field.ty, unit_name, own_types);
            }
        }
        Enum(enum_ty) => {
            for argument in &mut enum_ty.type_arguments {
                qualify_owned_type(argument, unit_name, own_types);
            }
            enum_ty.name = qualify_owned_name(&enum_ty.name, unit_name, own_types);
            for variant in &mut enum_ty.variants {
                for field in &mut variant.fields {
                    qualify_owned_type(&mut field.ty, unit_name, own_types);
                }
            }
        }
        Named(name) => *name = qualify_owned_name(name, unit_name, own_types),
        artifact::InterfaceType::Applied(name, arguments) => {
            *name = qualify_owned_name(name, unit_name, own_types);
            for argument in arguments {
                qualify_owned_type(argument, unit_name, own_types);
            }
        }
        GenericParameter(_) => {}
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
