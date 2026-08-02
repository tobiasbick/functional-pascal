//! Conversion between semantic types and persistent interface types.

use std::fmt;
use std::sync::Arc;

use fpas_unit::interface as artifact;

use crate::scope::{Symbol, SymbolKind as SemaSymbolKind};
use crate::types::{
    EnumTy, EnumVariantTy, EventTy, FunctionTy, GenericParamDef, MethodKind, ParamTy, ProcedureTy,
    PropertyTy, RecordTy, Ty, TypeConstraint,
};

/// A Sema type cannot be represented in a valid exported interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceConversionError {
    detail: String,
}

impl InterfaceConversionError {
    /// Create an interface conversion error with a compiler-facing detail message.
    pub(super) fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for InterfaceConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot persist semantic interface type: {}",
            self.detail
        )
    }
}

impl std::error::Error for InterfaceConversionError {}
/// Convert one persisted interface symbol into a semantic scope symbol.
pub(super) fn interface_symbol_to_sema(
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
                    backing_value: None,
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

/// Convert a semantic type to a persisted reference, naming owned records and enums.
pub(super) fn ty_to_interface_reference(
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
                })
            })
            .collect::<Result<_, InterfaceConversionError>>()?,
    })
}

fn constraint_to_interface(constraint: TypeConstraint) -> artifact::TypeConstraint {
    match constraint {
        TypeConstraint::Comparable => artifact::TypeConstraint::Comparable,
        TypeConstraint::Numeric => artifact::TypeConstraint::Numeric,
        TypeConstraint::Printable => artifact::TypeConstraint::Printable,
    }
}

fn constraint_from_interface(constraint: artifact::TypeConstraint) -> TypeConstraint {
    match constraint {
        artifact::TypeConstraint::Comparable => TypeConstraint::Comparable,
        artifact::TypeConstraint::Numeric => TypeConstraint::Numeric,
        artifact::TypeConstraint::Printable => TypeConstraint::Printable,
    }
}
