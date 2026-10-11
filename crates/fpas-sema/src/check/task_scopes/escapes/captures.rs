//! Implicit capture parameters of named nested routines and their lexical lookup.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use super::origins::{Binding, Frame, Origin, Owners, merge};
use super::walker::Walker;
use crate::scope::canonical_symbol_name;

impl Walker<'_, '_> {
    /// Seed captures as extra inputs and expose the enclosing routines' lexical names.
    pub(super) fn install_routine_captures(&mut self, index: usize) {
        let routine = &self.table.routines[index];
        self.escapes
            .resize(routine.params.len() + routine.captures.len(), false);
        if let Some(frame) = self.frames.last_mut() {
            for (capture, name) in routine.captures.iter().enumerate() {
                frame.bindings.insert(
                    canonical_symbol_name(name),
                    Binding {
                        owners: vec![Origin::Parameter(routine.params.len() + capture)],
                        outer: true,
                        depth: 0,
                    },
                );
            }
        }
        let mut parents = Vec::new();
        let mut parent = routine.parent;
        while let Some(index) = parent {
            parents.push(Frame {
                routines: self.table.routines[index].nested.clone(),
                ..Frame::default()
            });
            parent = self.table.routines[index].parent;
        }
        parents.reverse();
        parents.append(&mut self.frames);
        self.frames = parents;
    }

    /// Origins retained when a named nested routine is used as a callable value.
    pub(super) fn routine_capture_origins(&self, index: usize) -> Owners {
        let mut owners = Vec::new();
        for name in &self.table.routines[index].captures {
            merge(&mut owners, self.read(name));
        }
        owners
    }
}
