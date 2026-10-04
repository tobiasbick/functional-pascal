//! Serialization-oriented semantic type descriptions.

use super::GenericParameter;

/// Built-in constraint attached to a generic parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeConstraint {
    /// Values support structural equality.
    Equatable,
    /// Values support equality and ordering comparisons.
    Comparable,
    /// Values support arithmetic operations.
    Numeric,
    /// Values can be rendered as text.
    Printable,
}

/// One callable parameter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParameterType {
    /// Source spelling of the parameter name.
    pub name: String,
    /// Whether the parameter is passed as mutable `var`.
    pub mutable: bool,
    /// Resolved parameter type.
    pub ty: InterfaceType,
}

/// Function or procedure signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CallableType {
    /// Whether the function explicitly guarantees pure evaluation.
    pub pure: bool,
    /// Generic parameters in declaration order.
    pub type_parameters: Vec<GenericParameter>,
    /// Formal parameters in declaration order.
    pub parameters: Vec<ParameterType>,
    /// Function result, or `None` for a procedure.
    pub result: Option<Box<InterfaceType>>,
    /// Whether additional positional arguments are accepted.
    pub variadic: bool,
}

/// One record field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldType {
    /// Source spelling of the field.
    pub name: String,
    /// Resolved field type.
    pub ty: InterfaceType,
    /// Checked default used when the field is omitted from construction.
    pub default_value: Option<super::FieldDefault>,
}

/// Exported record layout and field visibility.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecordType {
    /// Canonical qualified record name.
    pub name: String,
    /// Generic parameters in declaration order.
    pub type_parameters: Vec<GenericParameter>,
    /// Arguments of a concrete or enclosing-generic application.
    pub type_arguments: Vec<InterfaceType>,
    /// Whether the nominal type designates an opaque host resource.
    pub is_resource: bool,
    /// Canonical source unit that owns private members.
    pub owner_unit: Option<String>,
    /// Names of record members not declared `public`.
    pub private_members: Vec<String>,
    /// Fields in layout order.
    pub fields: Vec<FieldType>,
}

/// One enum variant.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumVariant {
    /// Source spelling of the variant.
    pub name: String,
    /// Associated-data fields in declaration order.
    pub fields: Vec<FieldType>,
    /// Explicit backing value, when declared.
    pub backing_value: Option<i64>,
}

/// Exported enum layout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumType {
    /// Canonical qualified enum name.
    pub name: String,
    /// Generic parameters in declaration order.
    pub type_parameters: Vec<GenericParameter>,
    /// Arguments of a concrete or enclosing-generic application.
    pub type_arguments: Vec<InterfaceType>,
    /// Variants in declaration/backing-value order.
    pub variants: Vec<EnumVariant>,
}

/// Stable type language stored in a compiled-unit interface.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InterfaceType {
    /// Signed integer.
    Integer,
    /// IEEE real number.
    Real,
    /// Boolean.
    Boolean,
    /// UTF-8 string.
    String,
    /// Procedure result.
    Unit,
    /// Homogeneous array.
    Array(Box<Self>),
    /// Bounded FIFO channel element type.
    Channel(Box<Self>),
    /// Dictionary key and value types.
    Dictionary(Box<Self>, Box<Self>),
    /// Optional value.
    Option(Box<Self>),
    /// Success and error values.
    Result(Box<Self>, Box<Self>),
    /// Concurrent task result.
    Task(Box<Self>),
    /// Function signature.
    Function(CallableType),
    /// Procedure signature.
    Procedure(CallableType),
    /// Complete exported record descriptor.
    Record(Box<RecordType>),
    /// Complete exported enum descriptor.
    Enum(Box<EnumType>),
    /// Reference to a canonical named type, including recursive references.
    Named(String),
    /// Application of a canonical nominal generic type, including recursive references.
    Applied(String, Vec<Self>),
    /// Generic parameter with its resolved constraint.
    GenericParameter(GenericParameter),
}
