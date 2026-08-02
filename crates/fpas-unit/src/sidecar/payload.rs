//! Typed payload decoding and logical unit identity validation.

use std::collections::HashSet;

use crate::CompiledUnit;
use crate::interface::{UnitInterface, decode_interface};
use crate::object::{RelocatableObject, decode_object};

use super::{LoadedUnit, SidecarCorruption};

pub(super) fn validate(compiled: CompiledUnit) -> Result<LoadedUnit, SidecarCorruption> {
    let interface =
        decode_interface(&compiled.interface).map_err(|_| SidecarCorruption::InterfacePayload)?;
    let object = decode_object(&compiled.object).map_err(|_| SidecarCorruption::ObjectPayload)?;
    validate_identity(&compiled, &interface, &object)?;
    validate_symbols(&interface)?;
    Ok(LoadedUnit {
        compiled,
        interface,
        object,
    })
}

fn validate_identity(
    compiled: &CompiledUnit,
    interface: &UnitInterface,
    object: &RelocatableObject,
) -> Result<(), SidecarCorruption> {
    let envelope = &compiled.identity.unit_name;
    if !envelope.eq_ignore_ascii_case(&interface.unit_name) {
        return Err(SidecarCorruption::InterfaceUnitName {
            envelope: envelope.clone(),
            payload: interface.unit_name.clone(),
        });
    }
    if !envelope.eq_ignore_ascii_case(&object.owner) {
        return Err(SidecarCorruption::ObjectOwner {
            envelope: envelope.clone(),
            payload: object.owner.clone(),
        });
    }
    Ok(())
}

fn validate_symbols(interface: &UnitInterface) -> Result<(), SidecarCorruption> {
    let mut names = HashSet::with_capacity(interface.symbols.len());
    let mut qualified_names = HashSet::with_capacity(interface.symbols.len());
    for symbol in &interface.symbols {
        let name = symbol.name.to_ascii_lowercase();
        if !names.insert(name) {
            return Err(SidecarCorruption::DuplicateSymbol(symbol.name.clone()));
        }
        let qualified_name = symbol.qualified_name.to_ascii_lowercase();
        if !qualified_names.insert(qualified_name) {
            return Err(SidecarCorruption::DuplicateSymbol(
                symbol.qualified_name.clone(),
            ));
        }
        let expected = format!("{}.{}", interface.unit_name, symbol.name);
        if !symbol.qualified_name.eq_ignore_ascii_case(&expected) {
            return Err(SidecarCorruption::SymbolOwner {
                symbol: symbol.name.clone(),
                qualified_name: symbol.qualified_name.clone(),
                unit_name: interface.unit_name.clone(),
            });
        }
    }
    Ok(())
}
