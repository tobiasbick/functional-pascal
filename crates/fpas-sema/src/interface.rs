//! Conversion between Sema types and persistent compiled-unit interface types.

mod analysis;
mod conversion;
mod export;
mod install;
#[cfg(test)]
mod tests;

pub use analysis::{
    UnitAnalysis, analyze_program_with_interface_support, analyze_program_with_interfaces,
    analyze_unit, analyze_unit_with_interface_support,
};
pub use conversion::{InterfaceConversionError, interface_type_to_ty, ty_to_interface_type};
