//! Exported symbol and unit-interface descriptions.

use super::InterfaceType;

/// Compile-time value needed by a consuming unit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConstantValue {
    /// Signed integer value.
    Integer(i64),
    /// IEEE-754 bits of a real value.
    Real(u64),
    /// Boolean value.
    Boolean(bool),
    /// UTF-8 string value.
    String(String),
    /// Enum backing value.
    EnumValue {
        /// Canonical enum type name.
        enum_name: String,
        /// Canonical variant name.
        variant_name: String,
        /// Resolved backing value.
        backing_value: i64,
    },
}

/// Runtime and semantic category of an exported symbol.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolKind {
    /// Static constant; non-scalar values use immutable runtime global storage.
    Constant(Option<ConstantValue>),
    /// Static aggregate metadata accompanying immutable runtime global storage.
    AggregateConstant(super::StaticValue),
    /// Immutable module variable.
    Variable,
    /// Mutable module variable.
    MutableVariable,
    /// Function definition.
    Function,
    /// Procedure definition.
    Procedure,
    /// Named type definition or alias.
    Type,
    /// Simple enum member.
    EnumMember(ConstantValue),
    /// Associated-data enum constructor.
    EnumVariantConstructor,
}

/// One public symbol exported by a source unit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InterfaceSymbol {
    /// Public spelling used for a short import.
    pub name: String,
    /// Canonical fully qualified definition name.
    pub qualified_name: String,
    /// Resolved symbol type.
    pub ty: InterfaceType,
    /// Symbol category and compile-time value.
    pub kind: SymbolKind,
}

/// Complete public semantic surface of one source unit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitInterface {
    /// Canonical unit name.
    pub unit_name: String,
    /// Public symbols in canonical deterministic order.
    pub symbols: Vec<InterfaceSymbol>,
}

impl UnitInterface {
    /// Normalize nested type identities and sort unordered public members.
    ///
    /// Source spelling of unit and symbol names is retained for diagnostics.
    #[must_use]
    pub fn canonicalized(mut self) -> Self {
        for symbol in &mut self.symbols {
            canonicalize_type(&mut symbol.ty);
            canonicalize_symbol_kind(&mut symbol.kind);
        }
        self.symbols.sort_by(|left, right| {
            canonical_name(&left.name)
                .cmp(&canonical_name(&right.name))
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.qualified_name.cmp(&right.qualified_name))
        });
        self
    }
}

pub(super) fn canonical_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn canonicalize_type(ty: &mut InterfaceType) {
    use InterfaceType::{
        Array, Channel, Dictionary, Enum, Function, GenericParameter, Named, Option, Procedure,
        Record, Result, Task,
    };
    match ty {
        Array(inner) | Channel(inner) | Option(inner) | Task(inner) => canonicalize_type(inner),
        Dictionary(key, value) | Result(key, value) => {
            canonicalize_type(key);
            canonicalize_type(value);
        }
        Function(callable) | Procedure(callable) => canonicalize_callable(callable),
        Record(record) => {
            record.name = canonical_name(&record.name);
            for parameter in &mut record.type_parameters {
                parameter.canonicalize();
            }
            for argument in &mut record.type_arguments {
                canonicalize_type(argument);
            }
            record.owner_unit = record.owner_unit.as_deref().map(canonical_name);
            for member in &mut record.private_members {
                *member = canonical_name(member);
            }
            record.private_members.sort();
            record.private_members.dedup();
            for field in &mut record.fields {
                canonicalize_type(&mut field.ty);
                if let Some(value) = &mut field.default_value {
                    canonicalize_default(value);
                }
            }
        }
        Enum(enum_ty) => {
            enum_ty.name = canonical_name(&enum_ty.name);
            for parameter in &mut enum_ty.type_parameters {
                parameter.canonicalize();
            }
            for argument in &mut enum_ty.type_arguments {
                canonicalize_type(argument);
            }
            for variant in &mut enum_ty.variants {
                for field in &mut variant.fields {
                    canonicalize_type(&mut field.ty);
                    if let Some(value) = &mut field.default_value {
                        canonicalize_default(value);
                    }
                }
            }
        }
        Named(name) => *name = canonical_name(name),
        InterfaceType::Applied(name, arguments) => {
            *name = canonical_name(name);
            for argument in arguments {
                canonicalize_type(argument);
            }
        }
        GenericParameter(parameter) => parameter.canonicalize(),
        _ => {}
    }
}

fn canonicalize_symbol_kind(kind: &mut SymbolKind) {
    match kind {
        SymbolKind::Constant(Some(value)) | SymbolKind::EnumMember(value) => {
            canonicalize_constant(value);
        }
        SymbolKind::AggregateConstant(value) => value.canonicalize(),
        _ => {}
    }
}

/// Normalize nominal names in scalar enum metadata.
fn canonicalize_default(value: &mut super::FieldDefault) {
    match value {
        super::FieldDefault::Constant(value) => canonicalize_constant(value),
        super::FieldDefault::Initializer {
            name,
            pure_parameters,
        } => {
            *name = canonical_name(name);
            for parameter in pure_parameters.iter_mut() {
                parameter.unit = parameter.unit.as_deref().map(canonical_name);
            }
            pure_parameters.sort_by_key(|parameter| {
                (
                    parameter.unit.clone(),
                    parameter.source_id,
                    parameter.offset,
                )
            });
            pure_parameters.dedup();
        }
    }
}

pub(super) fn canonicalize_constant(value: &mut ConstantValue) {
    if let ConstantValue::EnumValue {
        enum_name,
        variant_name,
        ..
    } = value
    {
        *enum_name = canonical_name(enum_name);
        *variant_name = canonical_name(variant_name);
    }
}

fn canonicalize_callable(callable: &mut super::CallableType) {
    for parameter in &mut callable.type_parameters {
        parameter.canonicalize();
    }
    for parameter in &mut callable.parameters {
        canonicalize_type(&mut parameter.ty);
    }
    if let Some(result) = &mut callable.result {
        canonicalize_type(result);
    }
}
