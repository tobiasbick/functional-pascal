//! Interface lookup and identity tracking in deterministic unit order.

use std::collections::{BTreeMap, HashMap};

use fpas_program::LinkedUnitIdentity;
use fpas_unit::interface::UnitInterface;
use fpas_unit::{DependencyIdentity, Digest};

use crate::BuildEvent;

use super::CompiledUnits;

pub(super) fn direct_interfaces(
    uses: &[fpas_parser::Import],
    interfaces: &[UnitInterface],
    positions: &HashMap<String, usize>,
) -> Vec<UnitInterface> {
    uses.iter()
        .filter_map(|used| {
            positions
                .get(&used.unit.parts.join(".").to_ascii_lowercase())
                .map(|position| interfaces[*position].clone())
        })
        .collect()
}

#[derive(Default)]
pub(super) struct InterfaceRegistry {
    interfaces: Vec<UnitInterface>,
    positions: HashMap<String, usize>,
    hashes: HashMap<String, Digest>,
    dependencies: HashMap<String, Vec<DependencyIdentity>>,
}

#[cfg(test)]
thread_local! {
    /// Counts direct-interface clones on the build thread.
    pub(super) static DIRECT_INTERFACE_COPIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl InterfaceRegistry {
    pub(super) fn all(&self) -> &[UnitInterface] {
        &self.interfaces
    }

    /// Nested imported layouts and debug projections also consume transitive interfaces.
    pub(super) fn dependency_identities(
        &self,
        uses: &[fpas_parser::Import],
    ) -> Vec<DependencyIdentity> {
        let mut dependencies = BTreeMap::new();
        for used in uses {
            let name = canonical_unit_name(used);
            let Some(interface_hash) = self.hashes.get(&name) else {
                continue;
            };
            for dependency in self.dependencies.get(&name).into_iter().flatten() {
                dependencies.insert(dependency.unit_name.clone(), dependency.interface_hash);
            }
            dependencies.insert(name, *interface_hash);
        }
        dependencies
            .into_iter()
            .map(|(unit_name, interface_hash)| DependencyIdentity {
                unit_name,
                interface_hash,
            })
            .collect()
    }

    /// Copies direct interfaces only for units that require compilation.
    pub(super) fn direct_interfaces(&self, uses: &[fpas_parser::Import]) -> Vec<UnitInterface> {
        uses.iter()
            .filter_map(|used| {
                self.positions
                    .get(&canonical_unit_name(used))
                    .map(|position| {
                        #[cfg(test)]
                        DIRECT_INTERFACE_COPIES.with(|count| count.set(count.get() + 1));
                        self.interfaces[*position].clone()
                    })
            })
            .collect()
    }

    pub(super) fn insert(
        &mut self,
        name: String,
        interface: UnitInterface,
        hash: Digest,
        dependencies: Vec<DependencyIdentity>,
    ) {
        let position = self.interfaces.len();
        self.interfaces.push(interface);
        self.positions.insert(name.clone(), position);
        self.dependencies.insert(name.clone(), dependencies);
        self.hashes.insert(name, hash);
    }

    pub(super) fn finish<Object>(
        self,
        objects: Vec<Object>,
        linked_units: Vec<LinkedUnitIdentity>,
        events: Vec<BuildEvent>,
    ) -> CompiledUnits<Object> {
        CompiledUnits {
            objects,
            interfaces: self.interfaces,
            interface_positions: self.positions,
            events,
            linked_units,
        }
    }
}

fn canonical_unit_name(used: &fpas_parser::Import) -> String {
    used.unit.parts.join(".").to_ascii_lowercase()
}
