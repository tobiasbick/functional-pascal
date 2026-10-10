//! Serialization-oriented semantic type descriptions.

/// Built-in constraint attached to a generic parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeConstraint {
    /// Values support equality and ordering comparisons.
    Comparable,
    /// Values support arithmetic operations.
    Numeric,
    /// Values can be rendered as text.
    Printable,
}

/// One generic parameter in a callable, record, or enum declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenericParameter {
    /// Case-preserving parameter identity, qualified for record method parameters.
    pub name: String,
    /// Optional built-in constraint.
    pub constraint: Option<TypeConstraint>,
}

/// How a callable parameter receives its argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ParameterMode {
    /// Read-only value parameter.
    Value,
    /// `var` parameter that changes the caller's variable.
    Var,
}

/// One callable parameter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParameterType {
    /// Source spelling of the parameter name.
    pub name: String,
    /// Resolved parameter type.
    pub ty: InterfaceType,
    /// Parameter mode; part of callable compatibility.
    pub mode: ParameterMode,
}

/// Function or procedure signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CallableType {
    /// Generic parameters in declaration order.
    pub type_parameters: Vec<GenericParameter>,
    /// Formal parameters in declaration order.
    pub parameters: Vec<ParameterType>,
    /// Function result, or `None` for a procedure.
    pub result: Option<Box<InterfaceType>>,
    /// Whether additional positional arguments are accepted.
    pub variadic: bool,
}

/// An exported field's constant default.
///
/// **Documentation:** `docs/pascal/language/types/records.md`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FieldDefaultValue {
    /// Scalar constant expression.
    Scalar(super::ConstantValue),
    /// `None` for an optional field, including an optional callable.
    OptionNone,
}

/// One record field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldType {
    /// Source spelling of the field.
    pub name: String,
    /// Resolved field type.
    pub ty: InterfaceType,
    /// Canonical constant default, when the field may be omitted.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md`
    pub default_value: Option<FieldDefaultValue>,
}

/// An instance method and its callable signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MethodType {
    /// Static capture guarantees for this method and its result.
    #[serde(default)]
    pub discard: super::DiscardInfo,
    /// Source spelling of the method.
    pub name: String,
    /// Callable signature including the explicit `Self` parameter used internally.
    pub callable: CallableType,
}

/// Exported record layout and members.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecordType {
    /// Canonical qualified record name.
    pub name: String,
    /// Canonical source unit that owns private members.
    pub owner_unit: Option<String>,
    /// Generic parameters in declaration order.
    pub type_parameters: Vec<GenericParameter>,
    /// Actual arguments; empty for a generic declaration.
    pub type_arguments: Vec<InterfaceType>,
    /// Names of record members not declared `public`.
    pub private_members: Vec<String>,
    /// Fields in layout order.
    pub fields: Vec<FieldType>,
    /// Instance methods in canonical name order.
    pub methods: Vec<MethodType>,
    /// Static routines in canonical name order.
    pub static_routines: Vec<MethodType>,
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
    /// Actual arguments; empty for a declaration.
    pub type_arguments: Vec<InterfaceType>,
    /// Variants in declaration/backing-value order.
    pub variants: Vec<EnumVariant>,
}

/// Exported distinct domain type identity.
///
/// **Documentation:** `docs/pascal/language/types/distinct-types.md`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DistinctType {
    /// Source spelling of the declared type name.
    pub name: String,
    /// Canonical source unit that declared the type.
    pub owner_unit: Option<String>,
    /// Scalar underlying type.
    pub underlying: Box<InterfaceType>,
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
    /// Distinct domain type identity and underlying scalar type.
    Distinct(Box<DistinctType>),
    /// Reference to a canonical named type, including recursive references.
    Named(String),
    /// Reference to a generic record with concrete or enclosing generic arguments.
    Application {
        /// Canonical declaration name.
        name: String,
        /// Unit owning the declaration and its private members.
        owner_unit: Option<String>,
        /// Arguments in declaration order.
        arguments: Vec<Self>,
    },
    /// Reference to a generic enum without expanding recursive payloads.
    EnumApplication {
        /// Canonical declaration name.
        name: String,
        /// Arguments in declaration order.
        arguments: Vec<Self>,
    },
    /// Generic parameter with its resolved constraint.
    GenericParameter(String, Option<TypeConstraint>),
}
