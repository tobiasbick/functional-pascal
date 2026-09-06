//! Persistent compiled-unit interface types to semantic types.

use std::sync::Arc;

use fpas_unit::interface as artifact;

use crate::scope::{Symbol, SymbolKind as SemaSymbolKind};
use crate::types::{
    EnumTy, EnumVariantTy, EventTy, FunctionTy, GenericParamDef, MethodKind, ParamTy, ProcedureTy,
    PropertyTy, RecordTy, Ty, TypeConstraint,
};

use super::InterfaceConversionError;

/// Convert one persisted interface symbol into a semantic scope symbol.
pub(crate) fn interface_symbol_to_sema(
    exported: &artifact::InterfaceSymbol,
) -> Result<Symbol, InterfaceConversionError> {
    let (kind, mutable) = match &exported.kind {
        artifact::SymbolKind::Constant(_) => (SemaSymbolKind::Const, false),
        artifact::SymbolKind::Variable => (SemaSymbolKind::Var, false),
        artifact::SymbolKind::MutableVariable => (SemaSymbolKind::Var, true),
        artifact::SymbolKind::Function => (SemaSymbolKind::Function, false),
        artifact::SymbolKind::Procedure => (SemaSymbolKind::Procedure, false),
        artifact::SymbolKind::Type => (SemaSymbolKind::Type, false),
        artifact::SymbolKind::EnumMember(_) => (SemaSymbolKind::EnumMember, false),
        artifact::SymbolKind::EnumVariantConstructor => {
            (SemaSymbolKind::EnumVariantConstructor, false)
        }
    };
    Ok(Symbol {
        ty: interface_type_to_ty(&exported.ty)?,
        mutable,
        kind,
        task_bound: false,
    })
}

/// Reconstruct a Sema type from a compiled-unit representation.
pub fn interface_type_to_ty(ty: &artifact::InterfaceType) -> Result<Ty, InterfaceConversionError> {
    use artifact::InterfaceType as Input;
    Ok(match ty {
        Input::Integer => Ty::Integer,
        Input::Real => Ty::Real,
        Input::Boolean => Ty::Boolean,
        Input::String => Ty::String,
        Input::Unit => Ty::Unit,
        Input::Array(inner) => Ty::Array(Box::new(interface_type_to_ty(inner)?)),
        Input::Channel(inner) => Ty::Channel(Box::new(interface_type_to_ty(inner)?)),
        Input::Dictionary(key, value) => Ty::Dict(
            Box::new(interface_type_to_ty(key)?),
            Box::new(interface_type_to_ty(value)?),
        ),
        Input::Option(inner) => Ty::Option(Box::new(interface_type_to_ty(inner)?)),
        Input::Result(ok, error) => Ty::Result(
            Box::new(interface_type_to_ty(ok)?),
            Box::new(interface_type_to_ty(error)?),
        ),
        Input::Task(inner) => Ty::Task(Box::new(interface_type_to_ty(inner)?)),
        Input::Function(function) => Ty::Function(interface_to_function(function)?),
        Input::Procedure(procedure) => Ty::Procedure(interface_to_procedure(procedure)?),
        Input::Record(record) => Ty::Record(Arc::new(interface_to_record(record)?)),
        Input::Enum(enum_ty) => Ty::Enum(Arc::new(interface_to_enum(enum_ty)?)),
        Input::Named(name) => Ty::Named(name.clone()),
        Input::GenericParameter(name, constraint) => {
            Ty::GenericParam(name.clone(), constraint.map(constraint_from_interface))
        }
    })
}

fn interface_to_function(
    callable: &artifact::CallableType,
) -> Result<FunctionTy, InterfaceConversionError> {
    let Some(result) = &callable.result else {
        return Err(InterfaceConversionError::new(
            "a function signature has no result type",
        ));
    };
    Ok(FunctionTy {
        type_params: generic_parameters_from_interface(&callable.type_parameters),
        params: parameters_from_interface(&callable.parameters)?,
        return_type: Box::new(interface_type_to_ty(result)?),
        variadic: callable.variadic,
    })
}

fn interface_to_procedure(
    callable: &artifact::CallableType,
) -> Result<ProcedureTy, InterfaceConversionError> {
    if callable.result.is_some() {
        return Err(InterfaceConversionError::new(
            "a procedure signature unexpectedly has a result type",
        ));
    }
    Ok(ProcedureTy {
        type_params: generic_parameters_from_interface(&callable.type_parameters),
        params: parameters_from_interface(&callable.parameters)?,
        variadic: callable.variadic,
    })
}

fn parameters_from_interface(
    parameters: &[artifact::ParameterType],
) -> Result<Vec<ParamTy>, InterfaceConversionError> {
    parameters
        .iter()
        .map(|parameter| {
            Ok(ParamTy {
                mutable: parameter.mutable,
                name: parameter.name.clone(),
                ty: interface_type_to_ty(&parameter.ty)?,
            })
        })
        .collect()
}

fn generic_parameters_from_interface(
    parameters: &[artifact::GenericParameter],
) -> Vec<GenericParamDef> {
    parameters
        .iter()
        .map(|parameter| GenericParamDef {
            name: parameter.name.clone(),
            constraint: parameter.constraint.map(constraint_from_interface),
        })
        .collect()
}

fn interface_to_record(
    record: &artifact::RecordType,
) -> Result<RecordTy, InterfaceConversionError> {
    let mut methods = Vec::new();
    for method in &record.methods {
        methods.push((
            method.name.clone(),
            callable_to_method_kind(&method.callable)?,
        ));
    }
    let mut static_functions = Vec::new();
    let mut static_procedures = Vec::new();
    for routine in &record.static_routines {
        if routine.callable.result.is_some() {
            static_functions.push((
                routine.name.clone(),
                interface_to_function(&routine.callable)?,
            ));
        } else {
            static_procedures.push((
                routine.name.clone(),
                interface_to_procedure(&routine.callable)?,
            ));
        }
    }
    Ok(RecordTy {
        name: record.name.clone(),
        owner_unit: record.owner_unit.clone(),
        private_members: record.private_members.clone(),
        fields: record
            .fields
            .iter()
            .map(|field| Ok((field.name.clone(), interface_type_to_ty(&field.ty)?)))
            .collect::<Result<_, InterfaceConversionError>>()?,
        methods,
        static_functions,
        static_procedures,
        properties: record
            .properties
            .iter()
            .map(|property| {
                Ok((
                    property.name.clone(),
                    PropertyTy {
                        ty: interface_type_to_ty(&property.ty)?,
                        getter: property.getter.clone(),
                        setter: property.setter.clone(),
                    },
                ))
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
        events: record
            .events
            .iter()
            .map(|event| {
                Ok((
                    event.name.clone(),
                    EventTy {
                        handler_ty: interface_type_to_ty(&event.handler)?,
                        getter: event.getter.clone(),
                        setter: event.setter.clone(),
                        owner_unit: event.owner_unit.clone(),
                    },
                ))
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

fn callable_to_method_kind(
    callable: &artifact::CallableType,
) -> Result<MethodKind, InterfaceConversionError> {
    if callable.result.is_some() {
        Ok(MethodKind::Function(interface_to_function(callable)?))
    } else {
        Ok(MethodKind::Procedure(interface_to_procedure(callable)?))
    }
}

fn interface_to_enum(enum_ty: &artifact::EnumType) -> Result<EnumTy, InterfaceConversionError> {
    Ok(EnumTy {
        name: enum_ty.name.clone(),
        variants: enum_ty
            .variants
            .iter()
            .map(|variant| {
                Ok(EnumVariantTy {
                    name: variant.name.clone(),
                    fields: variant
                        .fields
                        .iter()
                        .map(|field| Ok((field.name.clone(), interface_type_to_ty(&field.ty)?)))
                        .collect::<Result<_, InterfaceConversionError>>()?,
                    backing_value: variant.backing_value,
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

fn constraint_from_interface(constraint: artifact::TypeConstraint) -> TypeConstraint {
    match constraint {
        artifact::TypeConstraint::Comparable => TypeConstraint::Comparable,
        artifact::TypeConstraint::Numeric => TypeConstraint::Numeric,
        artifact::TypeConstraint::Printable => TypeConstraint::Printable,
    }
}
