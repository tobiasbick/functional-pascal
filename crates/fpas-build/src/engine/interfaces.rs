//! Interface lookup and identity tracking in deterministic unit order.

use std::collections::HashMap;

use fpas_parser::QualifiedId;
use fpas_program::LinkedUnitIdentity;
use fpas_unit::interface::UnitInterface;
use fpas_unit::{DependencyIdentity, Digest};

use crate::BuildEvent;

use super::CompiledUnits;

pub(super) fn direct_interfaces_from_map(
    uses: &[QualifiedId],
    interfaces: &HashMap<String, UnitInterface>,
) -> Vec<UnitInterface> {
    uses.iter()
        .filter_map(|used| {
            interfaces
                .get(&used.parts.join(".").to_ascii_lowercase())
                .cloned()
        })
        .collect()
}

#[derive(Default)]
pub(super) struct InterfaceRegistry {
    names: Vec<String>,
    interfaces: Vec<UnitInterface>,
    positions: HashMap<String, usize>,
    hashes: HashMap<String, Digest>,
}

impl InterfaceRegistry {
    pub(super) fn all(&self) -> &[UnitInterface] {
        &self.interfaces
    }

    pub(super) fn direct_dependency_identities(
        &self,
        uses: &[QualifiedId],
    ) -> Vec<DependencyIdentity> {
        let mut dependencies = Vec::new();
        for used in uses {
            let name = canonical_unit_name(used);
            let Some(interface_hash) = self.hashes.get(&name) else {
                continue;
            };
            dependencies.push(DependencyIdentity {
                unit_name: name,
                interface_hash: *interface_hash,
            });
        }
        dependencies
    }

    pub(super) fn direct_interfaces(&self, uses: &[QualifiedId]) -> Vec<UnitInterface> {
        uses.iter()
            .filter_map(|used| {
                self.positions
                    .get(&canonical_unit_name(used))
                    .map(|position| self.interfaces[*position].clone())
            })
            .collect()
    }

    pub(super) fn insert(&mut self, name: String, interface: UnitInterface, hash: Digest) {
        let position = self.interfaces.len();
        self.names.push(name.clone());
        self.interfaces.push(interface);
        self.positions.insert(name.clone(), position);
        self.hashes.insert(name, hash);
    }

    pub(super) fn finish<Object>(
        self,
        objects: Vec<Object>,
        linked_units: Vec<LinkedUnitIdentity>,
        events: Vec<BuildEvent>,
    ) -> CompiledUnits<Object> {
        let interfaces = self
            .names
            .into_iter()
            .zip(self.interfaces.iter().cloned())
            .collect();
        CompiledUnits {
            objects,
            interfaces,
            events,
            linked_units,
            supporting_interfaces: self.interfaces,
        }
    }
}

fn canonical_unit_name(used: &QualifiedId) -> String {
    used.parts.join(".").to_ascii_lowercase()
}
