//! Semantic types to persistent compiled-unit interface types.

use fpas_unit::interface as artifact;

use crate::types::{
    EnumTy, FunctionTy, GenericParamDef, MethodKind, ParamTy, ProcedureTy, RecordTy, Ty,
    TypeConstraint,
};

use super::InterfaceConversionError;

/// Convert a resolved Sema type into its stable compiled-unit representation.
pub fn ty_to_interface_type(ty: &Ty) -> Result<artifact::InterfaceType, InterfaceConversionError> {
    use artifact::InterfaceType as Output;
    Ok(match ty {
        Ty::Integer => Output::Integer,
        Ty::Real => Output::Real,
        Ty::Boolean => Output::Boolean,
        Ty::String => Output::String,
        Ty::Unit => Output::Unit,
        Ty::Array(inner) => Output::Array(Box::new(ty_to_interface_type(inner)?)),
        Ty::Dict(key, value) => Output::Dictionary(
            Box::new(ty_to_interface_type(key)?),
            Box::new(ty_to_interface_type(value)?),
        ),
        Ty::Option(inner) => Output::Option(Box::new(ty_to_interface_type(inner)?)),
        Ty::Result(ok, error) => Output::Result(
            Box::new(ty_to_interface_type(ok)?),
            Box::new(ty_to_interface_type(error)?),
        ),
        Ty::Task(inner) => Output::Task(Box::new(ty_to_interface_type(inner)?)),
        Ty::Function(function) => Output::Function(function_to_interface(function)?),
        Ty::Procedure(procedure) => Output::Procedure(procedure_to_interface(procedure)?),
        Ty::Record(record) => Output::Record(Box::new(record_to_interface(record)?)),
        Ty::Enum(enum_ty) => Output::Enum(Box::new(enum_to_interface(enum_ty)?)),
        Ty::Named(name) => Output::Named(name.clone()),
        Ty::GenericParam(name, constraint) => {
            Output::GenericParameter(name.clone(), constraint.map(constraint_to_interface))
        }
        Ty::Error => {
            return Err(InterfaceConversionError::new(
                "error recovery placeholders are not exportable",
            ));
        }
    })
}

fn function_to_interface(
    function: &FunctionTy,
) -> Result<artifact::CallableType, InterfaceConversionError> {
    Ok(artifact::CallableType {
        type_parameters: generic_parameters_to_interface(&function.type_params),
        parameters: parameters_to_interface(&function.params)?,
        result: Some(Box::new(ty_to_interface_reference(&function.return_type)?)),
        variadic: function.variadic,
    })
}

fn procedure_to_interface(
    procedure: &ProcedureTy,
) -> Result<artifact::CallableType, InterfaceConversionError> {
    Ok(artifact::CallableType {
        type_parameters: generic_parameters_to_interface(&procedure.type_params),
        parameters: parameters_to_interface(&procedure.params)?,
        result: None,
        variadic: procedure.variadic,
    })
}

fn parameters_to_interface(
    parameters: &[ParamTy],
) -> Result<Vec<artifact::ParameterType>, InterfaceConversionError> {
    parameters
        .iter()
        .map(|parameter| {
            Ok(artifact::ParameterType {
                name: parameter.name.clone(),
                mutable: parameter.mutable,
                ty: ty_to_interface_reference(&parameter.ty)?,
            })
        })
        .collect()
}

fn generic_parameters_to_interface(
    parameters: &[GenericParamDef],
) -> Vec<artifact::GenericParameter> {
    parameters
        .iter()
        .map(|parameter| artifact::GenericParameter {
            name: parameter.name.clone(),
            constraint: parameter.constraint.map(constraint_to_interface),
        })
        .collect()
}

fn record_to_interface(
    record: &RecordTy,
) -> Result<artifact::RecordType, InterfaceConversionError> {
    Ok(artifact::RecordType {
        name: record.name.clone(),
        owner_unit: record.owner_unit.clone(),
        private_members: record.private_members.clone(),
        fields: record
            .fields
            .iter()
            .map(|(name, ty)| {
                Ok(artifact::FieldType {
                    name: name.clone(),
                    ty: ty_to_interface_reference(ty)?,
                    default_value: None,
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
        methods: record
            .methods
            .iter()
            .map(|(name, method)| method_to_interface(name, method))
            .collect::<Result<_, _>>()?,
        static_routines: record
            .static_functions
            .iter()
            .map(|(name, function)| {
                Ok(artifact::MethodType {
                    name: name.clone(),
                    callable: function_to_interface(function)?,
                })
            })
            .chain(record.static_procedures.iter().map(|(name, procedure)| {
                Ok(artifact::MethodType {
                    name: name.clone(),
                    callable: procedure_to_interface(procedure)?,
                })
            }))
            .collect::<Result<_, InterfaceConversionError>>()?,
        properties: record
            .properties
            .iter()
            .map(|(name, property)| {
                Ok(artifact::PropertyType {
                    name: name.clone(),
                    ty: ty_to_interface_reference(&property.ty)?,
                    getter: property.getter.clone(),
                    setter: property.setter.clone(),
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
        events: record
            .events
            .iter()
            .map(|(name, event)| {
                Ok(artifact::EventType {
                    name: name.clone(),
                    handler: ty_to_interface_reference(&event.handler_ty)?,
                    getter: event.getter.clone(),
                    setter: event.setter.clone(),
                    owner_unit: event.owner_unit.clone(),
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

fn method_to_interface(
    name: &str,
    method: &MethodKind,
) -> Result<artifact::MethodType, InterfaceConversionError> {
    let callable = match method {
        MethodKind::Function(function) => function_to_interface(function)?,
        MethodKind::Procedure(procedure) => procedure_to_interface(procedure)?,
    };
    Ok(artifact::MethodType {
        name: name.to_string(),
        callable,
    })
}

fn enum_to_interface(enum_ty: &EnumTy) -> Result<artifact::EnumType, InterfaceConversionError> {
    Ok(artifact::EnumType {
        name: enum_ty.name.clone(),
        variants: enum_ty
            .variants
            .iter()
            .map(|variant| {
                Ok(artifact::EnumVariant {
                    name: variant.name.clone(),
                    fields: variant
                        .fields
                        .iter()
                        .map(|(name, ty)| {
                            Ok(artifact::FieldType {
                                name: name.clone(),
                                ty: ty_to_interface_reference(ty)?,
                                default_value: None,
                            })
                        })
                        .collect::<Result<_, InterfaceConversionError>>()?,
                    backing_value: variant.backing_value,
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

/// Convert a semantic type to a persisted reference, naming owned records and enums.
pub(crate) fn ty_to_interface_reference(
    ty: &Ty,
) -> Result<artifact::InterfaceType, InterfaceConversionError> {
    use artifact::InterfaceType as Output;
    Ok(match ty {
        Ty::Record(record) => Output::Named(record.name.clone()),
        Ty::Enum(enum_ty) => Output::Named(enum_ty.name.clone()),
        Ty::Array(inner) => Output::Array(Box::new(ty_to_interface_reference(inner)?)),
        Ty::Dict(key, value) => Output::Dictionary(
            Box::new(ty_to_interface_reference(key)?),
            Box::new(ty_to_interface_reference(value)?),
        ),
        Ty::Option(inner) => Output::Option(Box::new(ty_to_interface_reference(inner)?)),
        Ty::Result(ok, error) => Output::Result(
            Box::new(ty_to_interface_reference(ok)?),
            Box::new(ty_to_interface_reference(error)?),
        ),
        Ty::Task(inner) => Output::Task(Box::new(ty_to_interface_reference(inner)?)),
        Ty::Function(function) => Output::Function(function_to_interface(function)?),
        Ty::Procedure(procedure) => Output::Procedure(procedure_to_interface(procedure)?),
        _ => ty_to_interface_type(ty)?,
    })
}

fn constraint_to_interface(constraint: TypeConstraint) -> artifact::TypeConstraint {
    match constraint {
        TypeConstraint::Comparable => artifact::TypeConstraint::Comparable,
        TypeConstraint::Numeric => artifact::TypeConstraint::Numeric,
        TypeConstraint::Printable => artifact::TypeConstraint::Printable,
    }
}
