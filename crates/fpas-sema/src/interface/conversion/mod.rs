//! Conversion between semantic types and persistent interface types.

mod from_interface;
mod to_interface;

use std::fmt;

pub(crate) use from_interface::interface_symbol_to_sema;
pub use from_interface::interface_type_to_ty;
pub(crate) use to_interface::ty_to_interface_reference;
pub use to_interface::ty_to_interface_type;

/// A Sema type cannot be represented in a valid exported interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceConversionError {
    detail: String,
}

impl InterfaceConversionError {
    /// Create an interface conversion error with a compiler-facing detail message.
    pub(super) fn new(detail: impl Into<String>) -> Self {
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
