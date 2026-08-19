//! Extraction and qualification of persistent interfaces from analyzed units.

use fpas_parser::{Decl, Expr, TypeBody, Unit, Visibility};
use fpas_unit::interface as artifact;

use crate::check;
use crate::scope::canonical_symbol_name;

use super::conversion::{
    InterfaceConversionError, ty_to_interface_reference, ty_to_interface_type,
};

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
            apply_declared_metadata(declaration, &mut ty)?;
            qualify_owned_type(&mut ty, &unit_name, &own_types);
            symbols.push(artifact::InterfaceSymbol {
                name: name.to_string(),
                qualified_name: format!("{unit_name}.{name}"),
                ty,
                kind: exported_symbol_kind(declaration),
            });
        }
        Ok(artifact::UnitInterface { unit_name, symbols }.canonicalized())
    }
}

fn apply_declared_metadata(
    declaration: &Decl,
    ty: &mut artifact::InterfaceType,
) -> Result<(), InterfaceConversionError> {
    let Decl::TypeDef(definition) = declaration else {
        return Ok(());
    };
    match (&definition.body, ty) {
        (TypeBody::Record(declared), artifact::InterfaceType::Record(interface)) => {
            for (field, declared_field) in interface.fields.iter_mut().zip(&declared.fields) {
                field.default_value = declared_field
                    .default_value
                    .as_ref()
                    .map(interface_constant_value)
                    .transpose()?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Return the declared source name of a top-level declaration.
pub(super) fn declaration_name(declaration: &Decl) -> &str {
    match declaration {
        Decl::Const(value) => &value.name,
        Decl::Var(value) | Decl::MutableVar(value) => &value.name,
        Decl::TypeDef(value) => &value.name,
        Decl::Function(value) => &value.name,
        Decl::Procedure(value) => &value.name,
    }
}

fn exported_symbol_kind(declaration: &Decl) -> artifact::SymbolKind {
    match declaration {
        Decl::Const(definition) => {
            artifact::SymbolKind::Constant(constant_value(&definition.value))
        }
        Decl::Var(_) => artifact::SymbolKind::Variable,
        Decl::MutableVar(_) => artifact::SymbolKind::MutableVariable,
        Decl::Function(_) => artifact::SymbolKind::Function,
        Decl::Procedure(_) => artifact::SymbolKind::Procedure,
        Decl::TypeDef(_) => artifact::SymbolKind::Type,
    }
}

fn constant_value(expression: &Expr) -> Option<artifact::ConstantValue> {
    match expression {
        Expr::Integer(value, _) => Some(artifact::ConstantValue::Integer(*value)),
        Expr::Real(value, _) => Some(artifact::ConstantValue::Real(value.to_bits())),
        Expr::Bool(value, _) => Some(artifact::ConstantValue::Boolean(*value)),
        Expr::Str(value, _) => Some(artifact::ConstantValue::String(value.clone())),
        Expr::Paren(inner, _) => constant_value(inner),
        Expr::UnaryOp {
            op: fpas_parser::UnaryOp::Negate,
            operand,
            ..
        } => match constant_value(operand)? {
            artifact::ConstantValue::Integer(value) => {
                value.checked_neg().map(artifact::ConstantValue::Integer)
            }
            artifact::ConstantValue::Real(bits) => Some(artifact::ConstantValue::Real(
                (-f64::from_bits(bits)).to_bits(),
            )),
            _ => None,
        },
        _ => None,
    }
}

fn interface_constant_value(
    expression: &Expr,
) -> Result<artifact::ConstantValue, InterfaceConversionError> {
    constant_value(expression).ok_or_else(|| {
        InterfaceConversionError::new(
            "exported record field defaults must be scalar constant expressions",
        )
    })
}

fn qualify_owned_type(
    ty: &mut artifact::InterfaceType,
    unit_name: &str,
    own_types: &std::collections::HashSet<String>,
) {
    use artifact::InterfaceType::{
        Array, Dictionary, Enum, Function, GenericParameter, Named, Option, Procedure, Record,
        Result, Task,
    };
    match ty {
        Array(inner) | Option(inner) | Task(inner) => {
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
            record.name = qualify_owned_name(&record.name, unit_name, own_types);
            record.owner_unit = Some(unit_name.to_string());
            for field in &mut record.fields {
                qualify_owned_type(&mut field.ty, unit_name, own_types);
            }
            for method in record
                .methods
                .iter_mut()
                .chain(record.static_routines.iter_mut())
            {
                qualify_callable(&mut method.callable, unit_name, own_types);
            }
            for property in &mut record.properties {
                qualify_owned_type(&mut property.ty, unit_name, own_types);
                property.getter = property
                    .getter
                    .take()
                    .map(|name| qualify_member_name(&name, unit_name, own_types));
                property.setter = property
                    .setter
                    .take()
                    .map(|name| qualify_member_name(&name, unit_name, own_types));
            }
            for event in &mut record.events {
                qualify_owned_type(&mut event.handler, unit_name, own_types);
                event.getter = qualify_member_name(&event.getter, unit_name, own_types);
                event.setter = qualify_member_name(&event.setter, unit_name, own_types);
                event.owner_unit = Some(unit_name.to_string());
            }
        }
        Enum(enum_ty) => {
            enum_ty.name = qualify_owned_name(&enum_ty.name, unit_name, own_types);
            for variant in &mut enum_ty.variants {
                for field in &mut variant.fields {
                    qualify_owned_type(&mut field.ty, unit_name, own_types);
                }
            }
        }
        Named(name) => *name = qualify_owned_name(name, unit_name, own_types),
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

fn qualify_member_name(
    name: &str,
    unit_name: &str,
    own_types: &std::collections::HashSet<String>,
) -> String {
    let owner = name.split('.').next().unwrap_or(name);
    if own_types.contains(&canonical_symbol_name(owner)) {
        format!("{unit_name}.{name}")
    } else {
        name.to_string()
    }
}
