//! SDK-derived declaration shapes shared by semantic registration and VM adapters.

use crate::schema::Version;
use serde_json::Value;
use std::collections::BTreeMap;

/// Detached protocol value type, without a compiler or VM dependency.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    /// Boolean scalar.
    Boolean,
    /// Checked signed or bounded unsigned integer.
    Integer {
        /// Inclusive lower bound.
        minimum: i64,
        /// Inclusive upper bound.
        maximum: i64,
    },
    /// Full unsigned range represented by the shared UInt64 record.
    UInt64,
    /// Finite binary64 scalar.
    Real,
    /// Ordinary text.
    String,
    /// Arbitrary JSON, optionally restricted to an object.
    Raw {
        /// Require an object rather than an arbitrary JSON value.
        object: bool,
    },
    /// Ordered values.
    Array(Box<Type>),
    /// Ordered string-keyed values.
    Dictionary(Box<Type>),
    /// Absence and null are equivalent.
    Option(Box<Type>),
    /// Absence, null and replacement are different.
    Field(Box<Type>),
    /// SDK or synthesized nominal declaration key.
    Named(String),
}

/// Location of a record field on the wire.
#[derive(Clone, Debug, PartialEq)]
pub enum WireField {
    /// Ordinary JSON property.
    Property(String),
    /// Flattened tagged/untagged payload.
    Flatten,
    /// Additional explicitly retained object members.
    Extra,
}

/// One ordered record field.
#[derive(Clone, Debug)]
pub struct Field {
    /// Keyword-safe FPAS field name.
    pub name: String,
    /// Wire property or flattening rule.
    pub wire: WireField,
    /// Concrete field type.
    pub ty: Type,
    /// Mandatory JSON property.
    pub required: bool,
    /// SDK default value when omitted.
    pub default: Option<Value>,
    /// Pinned SDK invalid-optional-field behavior.
    pub default_on_error: bool,
    /// Pinned SDK list-entry behavior.
    pub skip_invalid_items: bool,
}

/// One typed enum branch.
#[derive(Clone, Debug)]
pub struct Variant {
    /// Keyword-safe FPAS constructor name.
    pub name: String,
    /// Optional discriminator property and value, or a scalar literal.
    pub tag: Option<(Option<String>, Value)>,
    /// The SDK permits omission of this discriminator and supplies its default.
    pub default_tag: bool,
    /// Associated payload, absent for literal branches.
    pub payload: Option<Type>,
}

/// Unknown-variant representation.
#[derive(Clone, Debug)]
pub enum Unknown {
    /// Open string enum.
    String,
    /// Tagged payload with original JSON.
    Tagged(String),
}

/// Shape of a public protocol declaration.
#[derive(Clone, Debug)]
pub enum Shape {
    /// Scalar alias or nominal string ID.
    Scalar(Type),
    /// Ordered record layout.
    Record(Vec<Field>),
    /// Tagged or untagged data enum.
    Enum {
        /// Known branches in SDK declaration order.
        variants: Vec<Variant>,
        /// Forward-compatible representation, when applicable.
        unknown: Option<Unknown>,
    },
    /// SDK extension payload with its method outside the JSON body.
    Extension {
        /// Response bodies have no method field.
        response: bool,
    },
}

/// One complete declaration from the pinned SDK graph.
#[derive(Clone, Debug)]
pub struct Declaration {
    /// Stable graph key, including synthesized inline declarations.
    pub key: String,
    /// Qualified FPAS nominal name.
    pub name: String,
    /// Owning public unit.
    pub module: &'static str,
    /// Full value shape.
    pub shape: Shape,
    /// Exact SDK serde validator exists for this declaration.
    pub sdk: bool,
    /// Method metadata provided by the SDK schema.
    pub method: Option<String>,
}

/// Complete, versioned protocol declaration graph.
#[derive(Debug)]
pub struct Catalog {
    /// Selected protocol family.
    pub version: Version,
    /// All reachable declarations, including inline union payloads.
    pub declarations: BTreeMap<String, Declaration>,
}

/// SDK root families whose variants determine method and payload identity.
pub const ROOTS: [&str; 7] = [
    "ClientRequest",
    "AgentResponse",
    "ClientNotification",
    "AgentRequest",
    "ClientResponse",
    "AgentNotification",
    "ProtocolLevelNotification",
];
