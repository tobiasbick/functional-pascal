//! Capability flow snapshots preserve lexical binding identity across branches.

use std::collections::HashMap;
use std::sync::Arc;

use super::ScopeStack;
use crate::TransferCapability;

#[derive(Debug, Clone)]
struct BindingCapability {
    identity: Arc<()>,
    capability: TransferCapability,
}

#[derive(Debug, Clone)]
struct ScopeCapabilities {
    identity: Arc<()>,
    bindings: HashMap<String, BindingCapability>,
}

/// Flow state for the exact bindings that existed at a control-flow boundary.
#[derive(Debug, Clone)]
pub(crate) struct CapabilitySnapshot {
    scopes: Vec<ScopeCapabilities>,
}

impl PartialEq for CapabilitySnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.scopes.len() == other.scopes.len()
            && self.scopes.iter().zip(&other.scopes).all(|(left, right)| {
                Arc::ptr_eq(&left.identity, &right.identity)
                    && left.bindings.len() == right.bindings.len()
                    && left.bindings.iter().all(|(name, binding)| {
                        right.bindings.get(name).is_some_and(|other| {
                            Arc::ptr_eq(&binding.identity, &other.identity)
                                && binding.capability == other.capability
                        })
                    })
            })
    }
}

impl Eq for CapabilitySnapshot {}

/// Abrupt exits of one loop, kept separate from its normal backedge.
#[derive(Debug, Default)]
pub(crate) struct CapabilityLoop {
    pub(crate) breaks: Vec<CapabilitySnapshot>,
    pub(crate) continues: Vec<CapabilitySnapshot>,
}

impl ScopeStack {
    pub(crate) fn snapshot_capabilities(&self) -> CapabilitySnapshot {
        CapabilitySnapshot {
            scopes: self
                .scopes
                .iter()
                .map(|scope| ScopeCapabilities {
                    identity: Arc::clone(&scope.identity),
                    bindings: scope
                        .symbols
                        .iter()
                        .map(|(name, entry)| {
                            (
                                name.clone(),
                                BindingCapability {
                                    identity: Arc::clone(&entry.identity),
                                    capability: entry.symbol.capability,
                                },
                            )
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    pub(crate) fn restore_capabilities(&mut self, snapshot: &CapabilitySnapshot) {
        self.join_capability_snapshots(std::slice::from_ref(snapshot));
    }

    /// Join only matching live bindings; branch locals and shadowing cannot leak.
    pub(crate) fn join_capability_snapshots(&mut self, snapshots: &[CapabilitySnapshot]) {
        for scope in &mut self.scopes {
            for (name, entry) in &mut scope.symbols {
                let capability = snapshots.iter().filter_map(|snapshot| {
                    snapshot
                        .scopes
                        .iter()
                        .find(|saved| Arc::ptr_eq(&saved.identity, &scope.identity))
                        .and_then(|saved| saved.bindings.get(name))
                        .filter(|saved| Arc::ptr_eq(&saved.identity, &entry.identity))
                        .map(|saved| saved.capability)
                });
                if let Some(joined) = capability.reduce(TransferCapability::join) {
                    entry.symbol.capability = joined;
                }
            }
        }
    }

    pub(crate) fn begin_capability_loop(&mut self) {
        self.capability_loops.push(CapabilityLoop::default());
    }

    pub(crate) fn finish_capability_loop(&mut self) -> CapabilityLoop {
        self.capability_loops.pop().unwrap_or_default()
    }

    pub(crate) fn record_capability_loop_break(&mut self) {
        let snapshot = self.snapshot_capabilities();
        if let Some(current) = self.capability_loops.last_mut() {
            current.breaks.push(snapshot);
        }
    }

    pub(crate) fn record_capability_loop_continue(&mut self) {
        let snapshot = self.snapshot_capabilities();
        if let Some(current) = self.capability_loops.last_mut() {
            current.continues.push(snapshot);
        }
    }

    /// A repeat condition is reached by normal completion and every continue.
    pub(crate) fn join_capability_loop_continues(&mut self, include_normal: bool) -> bool {
        let mut exits = Vec::new();
        if include_normal {
            exits.push(self.snapshot_capabilities());
        }
        if let Some(current) = self.capability_loops.last() {
            exits.extend(current.continues.iter().cloned());
        }
        self.join_capability_snapshots(&exits);
        !exits.is_empty()
    }

    pub(crate) fn take_capability_loops(&mut self) -> Vec<CapabilityLoop> {
        std::mem::take(&mut self.capability_loops)
    }

    pub(crate) fn restore_capability_loops(&mut self, loops: Vec<CapabilityLoop>) {
        self.capability_loops = loops;
    }
}

#[cfg(test)]
mod tests;
