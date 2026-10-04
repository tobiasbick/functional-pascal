//! Persistent compiled-unit interface types to semantic types.

use std::sync::Arc;

use fpas_unit::interface as artifact;

use crate::scope::{Symbol, SymbolKind as SemaSymbolKind};
use crate::types::{
    EnumTy, EnumVariantTy, FunctionTy, GenericParamDef, ParamTy, ProcedureTy, RecordTy, Ty,
    TypeConstraint,
};

use super::InterfaceConversionError;

/// Convert one persisted interface symbol into a semantic scope symbol.
pub(crate) fn interface_symbol_to_sema(
    exported: &artifact::InterfaceSymbol,
) -> Result<Symbol, InterfaceConversionError> {
    let (kind, mutable) = match &exported.kind {
        artifact::SymbolKind::Constant(_) | artifact::SymbolKind::AggregateConstant(_) => {
            (SemaSymbolKind::Const, false)
        }
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
        Input::Applied(name, arguments) => Ty::Applied(
            name.clone(),
            arguments
                .iter()
                .map(interface_type_to_ty)
                .collect::<Result<_, _>>()?,
        ),
        Input::GenericParameter(parameter) => {
            Ty::GenericParam(Arc::new(generic_parameter_from_interface(parameter)))
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
        pure: callable.pure,
        type_params: generic_parameters_from_interface(&callable.type_parameters),
        params: parameters_from_interface(&callable.parameters)?,
        return_type: Box::new(interface_type_to_ty(result)?),
        variadic: callable.variadic,
    })
}

fn interface_to_procedure(
    callable: &artifact::CallableType,
) -> Result<ProcedureTy, InterfaceConversionError> {
    if callable.pure {
        return Err(InterfaceConversionError::new("a procedure cannot be pure"));
    }
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
        .map(generic_parameter_from_interface)
        .collect()
}

fn generic_parameter_from_interface(parameter: &artifact::GenericParameter) -> GenericParamDef {
    GenericParamDef {
        name: parameter.name.clone(),
        constraint: parameter.constraint.map(constraint_from_interface),
        identity: parameter.identity.clone(),
    }
}

fn interface_to_record(
    record: &artifact::RecordType,
) -> Result<RecordTy, InterfaceConversionError> {
    Ok(RecordTy {
        name: record.name.clone(),
        type_params: generic_parameters_from_interface(&record.type_parameters),
        type_args: record
            .type_arguments
            .iter()
            .map(interface_type_to_ty)
            .collect::<Result<_, _>>()?,
        is_resource: record.is_resource,
        owner_unit: record.owner_unit.clone(),
        private_members: record.private_members.clone(),
        fields: record
            .fields
            .iter()
            .map(|field| Ok((field.name.clone(), interface_type_to_ty(&field.ty)?)))
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

fn interface_to_enum(enum_ty: &artifact::EnumType) -> Result<EnumTy, InterfaceConversionError> {
    Ok(EnumTy {
        name: enum_ty.name.clone(),
        type_params: generic_parameters_from_interface(&enum_ty.type_parameters),
        type_args: enum_ty
            .type_arguments
            .iter()
            .map(interface_type_to_ty)
            .collect::<Result<_, _>>()?,
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
        artifact::TypeConstraint::Equatable => TypeConstraint::Equatable,
        artifact::TypeConstraint::Comparable => TypeConstraint::Comparable,
        artifact::TypeConstraint::Numeric => TypeConstraint::Numeric,
        artifact::TypeConstraint::Printable => TypeConstraint::Printable,
    }
}
