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

/// One generic parameter in a callable signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenericParameter {
    /// Source spelling of the parameter name.
    pub name: String,
    /// Optional built-in constraint.
    pub constraint: Option<TypeConstraint>,
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
    /// Canonical scalar default value, when the field may be omitted.
    pub default_value: Option<super::ConstantValue>,
}

/// An instance method and its callable signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MethodType {
    /// Source spelling of the method.
    pub name: String,
    /// Callable signature including the explicit `Self` parameter used internally.
    pub callable: CallableType,
}

/// A computed record property.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PropertyType {
    /// Source spelling of the property.
    pub name: String,
    /// Declared property type.
    pub ty: InterfaceType,
    /// Qualified getter definition, when readable.
    pub getter: Option<String>,
    /// Qualified setter definition, when writable.
    pub setter: Option<String>,
}

/// A record event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventType {
    /// Source spelling of the event.
    pub name: String,
    /// Declared handler callable type.
    pub handler: InterfaceType,
    /// Qualified getter definition.
    pub getter: String,
    /// Qualified setter definition.
    pub setter: String,
    /// Canonical unit owning the declaring record.
    pub owner_unit: Option<String>,
}

/// Exported record layout and members.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecordType {
    /// Canonical qualified record name.
    pub name: String,
    /// Canonical source unit that owns private members.
    pub owner_unit: Option<String>,
    /// Names of fields and routines declared `private`.
    pub private_members: Vec<String>,
    /// Fields in layout order.
    pub fields: Vec<FieldType>,
    /// Instance methods in canonical name order.
    pub methods: Vec<MethodType>,
    /// Static routines in canonical name order.
    pub static_routines: Vec<MethodType>,
    /// Properties in canonical name order.
    pub properties: Vec<PropertyType>,
    /// Events in canonical name order.
    pub events: Vec<EventType>,
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
    /// Generic parameter with its resolved constraint.
    GenericParameter(String, Option<TypeConstraint>),
}
