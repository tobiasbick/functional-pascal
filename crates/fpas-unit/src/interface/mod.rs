//! Stable semantic interfaces consumed without dependency implementation ASTs.

mod codec;
mod hash;
mod symbols;
mod types;

pub use codec::{InterfaceFormatError, decode_interface, encode_interface};
pub use symbols::{ConstantValue, InterfaceSymbol, SymbolKind, UnitInterface};
pub use types::{
    CallableType, EnumType, EnumVariant, EventType, FieldType, GenericParameter, InterfaceType,
    MethodType, ParameterType, PropertyType, RecordType, TypeConstraint,
};
