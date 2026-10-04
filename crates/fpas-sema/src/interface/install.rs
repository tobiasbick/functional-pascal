//! Installation of compiled-unit interface symbols into semantic scopes.

use fpas_parser::{Decl, Program};
use fpas_unit::interface as artifact;

use crate::check;
use crate::scope::{Symbol, SymbolKind as SemaSymbolKind};

use super::conversion::{InterfaceConversionError, interface_symbol_to_sema, interface_type_to_ty};

impl check::Checker {
    /// Install only qualified type definitions from transitive supporting interfaces.
    pub(crate) fn install_supporting_interface_types(
        &mut self,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        for interface in interfaces {
            self.supporting_unit_names
                .insert(interface.unit_name.to_ascii_lowercase());
            for exported in &interface.symbols {
                if exported.kind != artifact::SymbolKind::Type {
                    continue;
                }
                self.scopes.define_in_root(
                    &exported.qualified_name,
                    interface_symbol_to_sema(exported)?,
                );
                self.install_imported_record_defaults(&exported.ty);
            }
        }
        Ok(())
    }

    /// Install directly visible interfaces for one program.
    pub(crate) fn install_interfaces(
        &mut self,
        program: &Program,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        self.install_interfaces_for_declarations(&program.declarations, interfaces)
    }

    /// Install directly visible interface symbols alongside the given declarations.
    pub(crate) fn install_interfaces_for_declarations(
        &mut self,
        _declarations: &[Decl],
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        for interface in interfaces {
            for exported in &interface.symbols {
                let symbol = interface_symbol_to_sema(exported)?;
                self.scopes
                    .define_in_root(&exported.qualified_name, symbol.clone());
                self.install_imported_record_defaults(&exported.ty);
                self.install_imported_enum_variants(exported)?;
            }
        }

        Ok(())
    }

    fn install_imported_enum_variants(
        &mut self,
        exported: &artifact::InterfaceSymbol,
    ) -> Result<(), InterfaceConversionError> {
        let artifact::InterfaceType::Enum(enum_ty) = &exported.ty else {
            return Ok(());
        };
        let enum_symbol_ty = interface_type_to_ty(&exported.ty)?;
        for variant in &enum_ty.variants {
            let kind = if variant.fields.is_empty() {
                SemaSymbolKind::EnumMember
            } else {
                SemaSymbolKind::EnumVariantConstructor
            };
            let symbol = Symbol {
                ty: enum_symbol_ty.clone(),
                mutable: false,
                kind,
                task_bound: false,
            };
            let fully_qualified = format!("{}.{}", exported.qualified_name, variant.name);
            self.scopes.define_in_root(&fully_qualified, symbol);
        }
        Ok(())
    }
}
