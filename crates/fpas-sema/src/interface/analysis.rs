//! Semantic-analysis entry points backed by compiled-unit interfaces.

use fpas_parser::{Program, Unit};
use fpas_unit::interface as artifact;

use crate::{AnalysisMetadata, check};

use super::conversion::InterfaceConversionError;

/// Semantic result for one independently analyzed source unit.
pub struct UnitAnalysis {
    /// Compiler metadata keyed to the input unit AST.
    pub metadata: AnalysisMetadata,
    /// Canonical public interface extracted from the analyzed unit.
    pub interface: Option<artifact::UnitInterface>,
}

/// Analyze a program using dependency interfaces instead of dependency declarations.
pub fn analyze_program_with_interfaces(
    program: &Program,
    interfaces: &[artifact::UnitInterface],
) -> Result<AnalysisMetadata, InterfaceConversionError> {
    analyze_program_with_interface_support(program, interfaces, interfaces)
}

/// Analyze a program with directly visible interfaces plus transitive type support.
///
/// Supporting interfaces contribute only qualified type definitions. Their values and
/// callables do not become visible without a matching direct `uses` entry.
pub fn analyze_program_with_interface_support(
    program: &Program,
    interfaces: &[artifact::UnitInterface],
    supporting_interfaces: &[artifact::UnitInterface],
) -> Result<AnalysisMetadata, InterfaceConversionError> {
    let mut checker = check::Checker::new();
    checker.check_program_with_interfaces(program, interfaces, supporting_interfaces)?;
    Ok(checker.finish())
}

/// Analyze one source unit against dependency interfaces and extract its public interface.
pub fn analyze_unit(
    unit: &Unit,
    interfaces: &[artifact::UnitInterface],
) -> Result<UnitAnalysis, InterfaceConversionError> {
    analyze_unit_with_interface_support(unit, interfaces, interfaces)
}

/// Analyze one source unit with direct imports plus transitive qualified type support.
pub fn analyze_unit_with_interface_support(
    unit: &Unit,
    interfaces: &[artifact::UnitInterface],
    supporting_interfaces: &[artifact::UnitInterface],
) -> Result<UnitAnalysis, InterfaceConversionError> {
    let mut checker = check::Checker::new();
    checker.check_unit_with_interfaces(unit, interfaces, supporting_interfaces)?;
    let interface = if checker.errors.is_empty() {
        Some(checker.extract_unit_interface(unit)?)
    } else {
        None
    };
    Ok(UnitAnalysis {
        metadata: checker.finish(),
        interface,
    })
}
