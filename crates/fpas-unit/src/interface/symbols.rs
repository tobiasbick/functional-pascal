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
    /// Compile-time constant.
    Constant(Option<ConstantValue>),
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
        Array, Dictionary, Enum, Function, GenericParameter, Named, Option, Procedure, Record,
        Result, Task,
    };
    match ty {
        Array(inner) | Option(inner) | Task(inner) => canonicalize_type(inner),
        Dictionary(key, value) | Result(key, value) => {
            canonicalize_type(key);
            canonicalize_type(value);
        }
        Function(callable) | Procedure(callable) => canonicalize_callable(callable),
        Record(record) => {
            record.name = canonical_name(&record.name);
            record.owner_unit = record.owner_unit.as_deref().map(canonical_name);
            for member in &mut record.private_members {
                *member = canonical_name(member);
            }
            record.private_members.sort();
            record.private_members.dedup();
            for field in &mut record.fields {
                canonicalize_type(&mut field.ty);
                if let Some(value) = &mut field.default_value {
                    canonicalize_constant(value);
                }
            }
            for method in record
                .methods
                .iter_mut()
                .chain(record.static_routines.iter_mut())
            {
                canonicalize_callable(&mut method.callable);
            }
            for property in &mut record.properties {
                canonicalize_type(&mut property.ty);
                property.getter = property.getter.as_deref().map(canonical_name);
                property.setter = property.setter.as_deref().map(canonical_name);
            }
            for event in &mut record.events {
                canonicalize_type(&mut event.handler);
                event.getter = canonical_name(&event.getter);
                event.setter = canonical_name(&event.setter);
                event.owner_unit = event.owner_unit.as_deref().map(canonical_name);
            }
            sort_named(&mut record.methods, |value| &value.name);
            sort_named(&mut record.static_routines, |value| &value.name);
            sort_named(&mut record.properties, |value| &value.name);
            sort_named(&mut record.events, |value| &value.name);
        }
        Enum(enum_ty) => {
            enum_ty.name = canonical_name(&enum_ty.name);
            for variant in &mut enum_ty.variants {
                for field in &mut variant.fields {
                    canonicalize_type(&mut field.ty);
                    if let Some(value) = &mut field.default_value {
                        canonicalize_constant(value);
                    }
                }
            }
        }
        Named(name) => *name = canonical_name(name),
        GenericParameter(_, _) => {}
        _ => {}
    }
}

fn canonicalize_symbol_kind(kind: &mut SymbolKind) {
    match kind {
        SymbolKind::Constant(Some(value)) | SymbolKind::EnumMember(value) => {
            canonicalize_constant(value);
        }
        _ => {}
    }
}

fn canonicalize_constant(value: &mut ConstantValue) {
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
    for parameter in &mut callable.parameters {
        canonicalize_type(&mut parameter.ty);
    }
    if let Some(result) = &mut callable.result {
        canonicalize_type(result);
    }
}

fn sort_named<T>(values: &mut [T], name: impl Fn(&T) -> &str) {
    values.sort_by(|left, right| {
        canonical_name(name(left))
            .cmp(&canonical_name(name(right)))
            .then_with(|| name(left).cmp(name(right)))
    });
}
