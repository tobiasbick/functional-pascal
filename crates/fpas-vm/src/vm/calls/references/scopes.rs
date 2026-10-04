//! Sparse frame-owned leases independent from surviving register copies.

use std::sync::Arc;

use fpas_bytecode::SelectedReference;

#[derive(Default)]
/// Own leases independently from register copies and saved workers.
pub(in crate::vm) struct ReferenceScopes {
    depth: usize,
    scopes: Vec<Scope>,
}

struct Scope {
    depth: usize,
    leases: Vec<Arc<SelectedReference>>,
}

impl ReferenceScopes {
    /// Retain a reservation until callee entry or frame exit.
    pub(in crate::vm) fn reserve(&mut self, reference: Arc<SelectedReference>) {
        if self
            .scopes
            .last()
            .is_none_or(|scope| scope.depth != self.depth)
        {
            self.scopes.push(Scope {
                depth: self.depth,
                leases: Vec::new(),
            });
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.leases.push(reference);
        }
    }

    /// Remove completed authority so explicit-release loops retain bounded scope state.
    pub(in crate::vm) fn forget(&mut self, reference: &SelectedReference) {
        if let Some(scope) = self
            .scopes
            .last_mut()
            .filter(|scope| scope.depth == self.depth)
        {
            scope
                .leases
                .retain(|lease| !reference.same_authority(lease));
        }
    }

    /// Transfer matching reservations and own the activated callee parameters.
    pub(in crate::vm) fn enter(&mut self, parameters: Vec<Arc<SelectedReference>>) {
        if let Some(scope) = self
            .scopes
            .last_mut()
            .filter(|scope| scope.depth == self.depth)
        {
            // Entry transfers reservations; the caller may keep invalidated register copies.
            scope
                .leases
                .retain(|lease| !parameters.iter().any(|item| item.same_authority(lease)));
        }
        self.depth += 1;
        for parameter in parameters {
            self.reserve(parameter);
        }
    }

    /// Release the completed invocation and restore its caller's authority.
    pub(in crate::vm) fn leave(&mut self) {
        self.release_at(self.depth);
        self.depth = self.depth.saturating_sub(1);
    }

    /// Release discarded frames from youngest to oldest during debugger unwind.
    pub(in crate::vm) fn truncate(&mut self, retained_frames: usize) {
        while self.depth >= retained_frames {
            self.release_at(self.depth);
            if self.depth == 0 {
                break;
            }
            self.depth -= 1;
        }
    }

    /// Release every invocation on failure, cancellation or abandoned saved state.
    pub(in crate::vm) fn clear(&mut self) {
        while let Some(scope) = self.scopes.pop() {
            release(scope);
        }
        self.depth = 0;
    }

    /// Discard younger frames and the selected ordinary frame's reservations.
    pub(in crate::vm) fn restart(&mut self, depth: usize) {
        self.truncate(depth + 1);
        self.release_at(depth);
    }

    /// Whether frame reuse would discard authority owned by this invocation.
    pub(in crate::vm) fn current_has_leases(&self) -> bool {
        self.scopes
            .last()
            .is_some_and(|scope| scope.depth == self.depth && !scope.leases.is_empty())
    }

    fn release_at(&mut self, depth: usize) {
        if self.scopes.last().is_some_and(|scope| scope.depth == depth)
            && let Some(scope) = self.scopes.pop()
        {
            release(scope);
        }
    }
}

fn release(scope: Scope) {
    for lease in scope.leases.into_iter().rev() {
        let _ = lease.release();
    }
}

impl Drop for ReferenceScopes {
    fn drop(&mut self) {
        self.clear();
    }
}
