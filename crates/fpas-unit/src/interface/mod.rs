//! Stable semantic interfaces consumed without dependency implementation ASTs.

mod codec;
mod contracts;
mod hash;
mod record_constants;
mod symbols;
mod types;

pub use codec::{InterfaceFormatError, decode_interface, encode_interface};
pub use contracts::RoutineContract;
pub use record_constants::{RecordConstant, RecordConstantField};
pub use symbols::{ConstantValue, DiscardInfo, InterfaceSymbol, SymbolKind, UnitInterface};
pub use types::{
    CallableType, DistinctType, EnumType, EnumVariant, FieldDefaultValue, FieldType,
    GenericParameter, InterfaceType, MethodType, ParameterMode, ParameterType, RecordType,
    TypeConstraint,
};
