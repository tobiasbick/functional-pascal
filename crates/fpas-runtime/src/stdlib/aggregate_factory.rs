//! Runtime-owned construction of native record and enum values.

use crate::stdlib::StdError;
use crate::{SourceLocation, Value};

/// Constructs aggregate values against the layouts of the executing program.
pub trait AggregateFactory {
    /// Construct a native operational error by its shared catalog slot.
    ///
    /// # Errors
    /// Returns an internal diagnostic for missing or invalid layout metadata.
    fn native_error(
        &self,
        family: crate::stdlib::native_errors::NativeErrorFamily,
        variant: usize,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, StdError> {
        let variant = family.variants().get(variant).ok_or_else(|| {
            crate::stdlib::error::std_internal_error(
                "Native error catalog slot is unavailable",
                "Rebuild the runtime and compiler together.",
                location,
            )
        })?;
        self.enumeration(family.name(), variant.name, values, location)
    }

    /// Construct one record from values in declaration order.
    ///
    /// # Errors
    ///
    /// Returns an internal runtime diagnostic when the type or field layout is unavailable.
    fn record(
        &self,
        type_name: &str,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, StdError>;

    /// Construct one enum variant from associated values in declaration order.
    ///
    /// # Errors
    ///
    /// Returns an internal runtime diagnostic when the type, variant, or field layout is unavailable.
    fn enumeration(
        &self,
        type_name: &str,
        variant: &str,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, StdError>;
}
