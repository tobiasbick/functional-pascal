//! Conversion between Sema types and persistent compiled-unit interface types.

use std::fmt;
use std::sync::Arc;

use fpas_parser::{Decl, Expr, Program, TypeBody, Unit, Visibility};
use fpas_unit::interface as artifact;

use crate::scope::{Symbol, SymbolKind as SemaSymbolKind, canonical_symbol_name};
use crate::types::{
    EnumTy, EnumVariantTy, EventTy, FunctionTy, GenericParamDef, MethodKind, ParamTy, ProcedureTy,
    PropertyTy, RecordTy, Ty, TypeConstraint,
};
use crate::{SemaError, check};

/// A Sema type cannot be represented in a valid exported interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceConversionError {
    detail: String,
}

impl InterfaceConversionError {
    fn new(detail: impl Into<String>) -> Self {
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

/// All compiler-facing metadata produced by semantic analysis.
pub type AnalysisMetadata = (
    Vec<SemaError>,
    crate::ExprTypeMap,
    crate::MethodCallMap,
    crate::RecordDefaultsMap,
    crate::ScalarCaseBindingMap,
    crate::ClosureInfoMap,
    crate::NestedRoutineCaptureMap,
    crate::BoundMethodMap,
    crate::PropertyReadMap,
    crate::PropertyWriteMap,
    crate::EventWriteMap,
    crate::EventAssignedMap,
    crate::EventRaiseMap,
);

/// Semantic result for one independently analyzed source unit.
pub struct UnitAnalysis {
    /// Compiler metadata keyed to the input unit AST.
    pub metadata: AnalysisMetadata,
    /// Canonical public interface extracted from the analyzed unit.
    pub interface: Option<artifact::UnitInterface>,
}

/// Analyze a program using dependency interfaces instead of dependency declarations.
pub fn analyze_program_with_interfaces(
    program: &Program,
    interfaces: &[artifact::UnitInterface],
) -> Result<AnalysisMetadata, InterfaceConversionError> {
    analyze_program_with_interface_support(program, interfaces, interfaces)
}

/// Analyze a program with directly visible interfaces plus transitive type support.
///
/// Supporting interfaces contribute only qualified type definitions. Their values and
/// callables do not become visible without a matching direct `uses` entry.
pub fn analyze_program_with_interface_support(
    program: &Program,
    interfaces: &[artifact::UnitInterface],
    supporting_interfaces: &[artifact::UnitInterface],
) -> Result<AnalysisMetadata, InterfaceConversionError> {
    let mut checker = check::Checker::new();
    checker.check_program_with_interfaces(program, interfaces, supporting_interfaces)?;
    Ok(checker.finish())
}

/// Analyze one source unit against dependency interfaces and extract its public interface.
pub fn analyze_unit(
    unit: &Unit,
    interfaces: &[artifact::UnitInterface],
) -> Result<UnitAnalysis, InterfaceConversionError> {
    analyze_unit_with_interface_support(unit, interfaces, interfaces)
}

/// Analyze one source unit with direct imports plus transitive qualified type support.
pub fn analyze_unit_with_interface_support(
    unit: &Unit,
    interfaces: &[artifact::UnitInterface],
    supporting_interfaces: &[artifact::UnitInterface],
) -> Result<UnitAnalysis, InterfaceConversionError> {
    let mut checker = check::Checker::new();
    checker.check_unit_with_interfaces(unit, interfaces, supporting_interfaces)?;
    let interface = if checker.errors.is_empty() {
        Some(checker.extract_unit_interface(unit)?)
    } else {
        None
    };
    Ok(UnitAnalysis {
        metadata: checker.finish(),
        interface,
    })
}

impl check::Checker {
    pub(crate) fn install_supporting_interface_types(
        &mut self,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        for interface in interfaces {
            for exported in &interface.symbols {
                if exported.kind != artifact::SymbolKind::Type {
                    continue;
                }
                self.scopes.define_in_root(
                    &exported.qualified_name,
                    interface_symbol_to_sema(exported)?,
                );
            }
        }
        Ok(())
    }

    pub(crate) fn install_interfaces(
        &mut self,
        program: &Program,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        self.install_interfaces_for_declarations(&program.declarations, interfaces)
    }

    pub(crate) fn install_interfaces_for_declarations(
        &mut self,
        declarations: &[Decl],
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        use std::collections::{HashMap, HashSet};

        let own_names: HashSet<String> = declarations
            .iter()
            .map(declaration_name)
            .map(canonical_symbol_name)
            .collect();
        let mut short_candidates = HashMap::<String, Vec<(String, Symbol)>>::new();

        for interface in interfaces {
            for exported in &interface.symbols {
                let symbol = interface_symbol_to_sema(exported)?;
                self.scopes
                    .define_in_root(&exported.qualified_name, symbol.clone());
                self.install_imported_record_defaults(exported);
                if !own_names.contains(&canonical_symbol_name(&exported.name)) {
                    short_candidates
                        .entry(canonical_symbol_name(&exported.name))
                        .or_default()
                        .push((exported.qualified_name.clone(), symbol));
                }
                self.install_imported_enum_variants(exported, &mut short_candidates, &own_names)?;
            }
        }

        for (short, mut candidates) in short_candidates {
            candidates.sort_by(|left, right| {
                canonical_symbol_name(&left.0)
                    .cmp(&canonical_symbol_name(&right.0))
                    .then_with(|| left.0.cmp(&right.0))
            });
            candidates.dedup_by(|left, right| left.0.eq_ignore_ascii_case(&right.0));
            if candidates.len() == 1 {
                if let Some((_, symbol)) = candidates.pop() {
                    self.scopes.define_in_root(&short, symbol);
                }
            } else {
                self.ambiguous_imports.insert(
                    short,
                    candidates
                        .into_iter()
                        .map(|(qualified, _)| qualified)
                        .collect(),
                );
            }
        }
        Ok(())
    }

    fn install_imported_record_defaults(&mut self, exported: &artifact::InterfaceSymbol) {
        let artifact::InterfaceType::Record(record) = &exported.ty else {
            return;
        };
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
                    field.default_value.as_ref().map(constant_value_to_expr),
                )
            })
            .collect();
        self.record_defaults.insert(record.name.clone(), fields);
    }

    fn install_imported_enum_variants(
        &mut self,
        exported: &artifact::InterfaceSymbol,
        short_candidates: &mut std::collections::HashMap<String, Vec<(String, Symbol)>>,
        own_names: &std::collections::HashSet<String>,
    ) -> Result<(), InterfaceConversionError> {
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
                ty: enum_symbol_ty.clone(),
                mutable: false,
                kind,
                task_bound: false,
            };
            let fully_qualified = format!("{}.{}", enum_ty.name, variant.name);
            self.scopes.define_in_root(&fully_qualified, symbol.clone());
            if !own_names.contains(&canonical_symbol_name(&exported.name)) {
                self.scopes.define_in_root(
                    &format!("{}.{}", exported.name, variant.name),
                    symbol.clone(),
                );
            }
            let short = canonical_symbol_name(&variant.name);
            if !own_names.contains(&short) {
                short_candidates
                    .entry(short)
                    .or_default()
                    .push((fully_qualified, symbol));
            }
        }
        Ok(())
    }

    fn extract_unit_interface(
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
        (TypeBody::Enum(declared), artifact::InterfaceType::Enum(interface)) => {
            let mut next_value = 0_i64;
            for (variant, member) in interface.variants.iter_mut().zip(&declared.members) {
                let value = member.value.unwrap_or(next_value);
                variant.backing_value = Some(value);
                next_value = value.saturating_add(1);
            }
        }
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

fn declaration_name(declaration: &Decl) -> &str {
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

fn interface_symbol_to_sema(
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

fn ty_to_interface_reference(ty: &Ty) -> Result<artifact::InterfaceType, InterfaceConversionError> {
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

#[cfg(test)]
mod tests {
    use fpas_parser::{CompilationUnit, parse_compilation_unit};
    use fpas_unit::interface::{InterfaceType, SymbolKind};

    use super::analyze_unit;

    fn parse_unit(source: &str) -> fpas_parser::Unit {
        let (parsed, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "unexpected parse errors: {errors:#?}");
        let CompilationUnit::Unit(unit) = parsed else {
            panic!("fixture must parse as a unit");
        };
        unit
    }

    #[test]
    fn unit_interface_exports_public_symbols_and_qualified_types() {
        let unit = parse_unit(
            "unit Demo.Types;
             const Answer: integer := 42;
             private const Secret: integer := 7;
             type Point = record X: integer; Y: integer; end;
             function GetX(P: Point): integer;
             begin return P.X end;",
        );

        let analysis = analyze_unit(&unit, &[]).expect("unit analysis must succeed");
        assert!(analysis.metadata.0.is_empty(), "{:#?}", analysis.metadata.0);
        let interface = analysis.interface.expect("valid interface");
        assert_eq!(
            interface
                .symbols
                .iter()
                .map(|symbol| symbol.name.as_str())
                .collect::<Vec<_>>(),
            ["Answer", "GetX", "Point"]
        );
        assert!(
            interface
                .symbols
                .iter()
                .all(|symbol| !symbol.name.eq_ignore_ascii_case("Secret"))
        );
        let point = interface
            .symbols
            .iter()
            .find(|symbol| symbol.name == "Point")
            .expect("Point export");
        let InterfaceType::Record(record) = &point.ty else {
            panic!("Point must remain a record");
        };
        assert_eq!(record.name, "demo.types.point");
        assert_eq!(
            interface.symbols[0].kind,
            SymbolKind::Constant(Some(fpas_unit::interface::ConstantValue::Integer(42)))
        );
    }

    #[test]
    fn consumer_analysis_uses_interface_without_dependency_ast() {
        let dependency = parse_unit(
            "unit Demo.Api;
             type State = enum Idle; Ready; end;
             function Next(Value: integer): integer;
             begin return Value + 1 end;",
        );
        let dependency_analysis =
            analyze_unit(&dependency, &[]).expect("dependency analysis must succeed");
        assert!(
            dependency_analysis.metadata.0.is_empty(),
            "{:#?}",
            dependency_analysis.metadata.0
        );

        let consumer = parse_unit(
            "unit Demo.Consumer;
             uses Demo.Api;
             function Run(Value: integer): integer;
             begin
               var Current: State := State.Ready;
               return Next(Value)
             end;",
        );
        let consumer_analysis = analyze_unit(
            &consumer,
            &[dependency_analysis.interface.expect("dependency interface")],
        )
        .expect("consumer analysis must succeed");
        assert!(
            consumer_analysis.metadata.0.is_empty(),
            "{:#?}",
            consumer_analysis.metadata.0
        );
    }

    #[test]
    fn private_body_changes_do_not_change_interface_digest() {
        let left = parse_unit(
            "unit Demo.Stable;
             function PublicValue(X: integer): integer;
             begin return X end;
             private function Hidden(): integer;
             begin return 1 end;",
        );
        let right = parse_unit(
            "unit Demo.Stable;
             function PublicValue(X: integer): integer;
             begin return X + 99 end;
             private function Hidden(): integer;
             begin return 2 end;",
        );
        let left_interface = analyze_unit(&left, &[])
            .expect("left analysis")
            .interface
            .expect("left interface");
        let right_interface = analyze_unit(&right, &[])
            .expect("right analysis")
            .interface
            .expect("right interface");
        assert_eq!(
            left_interface.digest().expect("left digest"),
            right_interface.digest().expect("right digest")
        );
    }

    #[test]
    fn imported_name_ambiguity_is_reported_only_when_short_name_is_used() {
        let first = parse_unit(
            "unit Demo.First;
             function Value(): integer;
             begin return 1 end;",
        );
        let second = parse_unit(
            "unit Demo.Second;
             function Value(): integer;
             begin return 2 end;",
        );
        let interfaces = [
            analyze_unit(&first, &[])
                .expect("first analysis")
                .interface
                .expect("first interface"),
            analyze_unit(&second, &[])
                .expect("second analysis")
                .interface
                .expect("second interface"),
        ];

        let qualified = parse_unit(
            "unit Demo.Qualified;
             uses Demo.First, Demo.Second;
             function Run(): integer;
             begin return Demo.First.Value() + Demo.Second.Value() end;",
        );
        let qualified_analysis = analyze_unit(&qualified, &interfaces).expect("qualified analysis");
        assert!(
            qualified_analysis.metadata.0.is_empty(),
            "{:#?}",
            qualified_analysis.metadata.0
        );

        let ambiguous = parse_unit(
            "unit Demo.Ambiguous;
             uses Demo.First, Demo.Second;
             function Run(): integer;
             begin return Value() end;",
        );
        let ambiguous_analysis = analyze_unit(&ambiguous, &interfaces).expect("ambiguous analysis");
        assert_eq!(ambiguous_analysis.metadata.0.len(), 1);
        assert!(
            ambiguous_analysis.metadata.0[0]
                .message
                .contains("Ambiguous imported symbol `Value`")
        );
        assert!(ambiguous_analysis.interface.is_none());
    }
}
