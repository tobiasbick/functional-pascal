//! Stable semantic interfaces consumed without dependency implementation ASTs.

mod codec;
mod generics;
mod hash;
mod record_defaults;
mod symbols;
mod types;
mod values;

pub use codec::{InterfaceFormatError, decode_interface, encode_interface};
pub use generics::{GenericParameter, GenericParameterId};
pub use record_defaults::{FieldDefault, record_default_initializer};
pub use symbols::{ConstantValue, InterfaceSymbol, SymbolKind, UnitInterface};
pub use types::{
    CallableType, EnumType, EnumVariant, FieldType, InterfaceType, ParameterType, RecordType,
    TypeConstraint,
};
pub use values::StaticValue;
