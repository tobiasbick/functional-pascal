//! Rust-internal view registry for the TUI application framework (Phase 7).
//!
//! View handles are opaque identifiers maintained entirely by the host; FPAS has no
//! surface for them yet. Child ordering, focus chains, and FP-visible bindings are
//! added in later Phase 7 steps.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

/// Opaque handle identifying a host-managed view.
///
/// FPAS has no surface for this yet. Will be exposed as a first-class type in a
/// later Phase 7 step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(u32);

/// Axis-aligned bounding box for a view (terminal cell coordinates, origin top-left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

#[derive(Debug)]
struct ViewEntry {
    id: ViewId,
    rect: ViewRect,
}

/// Host-side registry for all active views in a TUI session.
///
/// Allocates monotonically increasing [`ViewId`]s. Registration order determines
/// initial paint order (index 0 = bottom / background).
#[derive(Debug, Default)]
pub struct ViewRegistry {
    next_id: u32,
    views: Vec<ViewEntry>,
}

impl ViewRegistry {
    /// Register a new view covering `rect`. Returns its opaque [`ViewId`].
    pub fn register(&mut self, rect: ViewRect) -> ViewId {
        let id = ViewId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.views.push(ViewEntry { id, rect });
        id
    }

    /// Remove a view by id. No-op if not found.
    pub fn unregister(&mut self, id: ViewId) {
        self.views.retain(|v| v.id != id);
    }

    /// Bounding rect for a view, or `None` if not registered.
    #[must_use]
    pub fn rect(&self, id: ViewId) -> Option<ViewRect> {
        self.views.iter().find(|v| v.id == id).map(|v| v.rect)
    }

    /// Update the bounding rect for a registered view. No-op if not found.
    pub fn set_rect(&mut self, id: ViewId, rect: ViewRect) {
        if let Some(entry) = self.views.iter_mut().find(|v| v.id == id) {
            entry.rect = rect;
        }
    }

    /// All registered view IDs, in registration order (bottom to top).
    pub fn ids(&self) -> impl Iterator<Item = ViewId> + '_ {
        self.views.iter().map(|v| v.id)
    }

    /// Count of registered views.
    #[must_use]
    pub fn len(&self) -> usize {
        self.views.len()
    }

    /// True when no views are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    /// Remove all registered views (called on `Application.Close`).
    pub fn clear(&mut self) {
        self.views.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i64, y: i64, w: i64, h: i64) -> ViewRect {
        ViewRect { x, y, width: w, height: h }
    }

    #[test]
    fn register_returns_distinct_ids() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(10, 0, 20, 5));
        assert_ne!(a, b);
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn rect_lookup_round_trips() {
        let mut reg = ViewRegistry::default();
        let r = rect(3, 7, 40, 20);
        let id = reg.register(r);
        assert_eq!(reg.rect(id), Some(r));
    }

    #[test]
    fn unregister_removes_view() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 10, 5));
        let b = reg.register(rect(0, 5, 10, 5));
        reg.unregister(a);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.rect(a), None);
        assert!(reg.rect(b).is_some());
    }

    #[test]
    fn set_rect_updates_bounds() {
        let mut reg = ViewRegistry::default();
        let id = reg.register(rect(0, 0, 10, 5));
        let new_r = rect(1, 2, 80, 24);
        reg.set_rect(id, new_r);
        assert_eq!(reg.rect(id), Some(new_r));
    }

    #[test]
    fn ids_preserves_registration_order() {
        let mut reg = ViewRegistry::default();
        let a = reg.register(rect(0, 0, 1, 1));
        let b = reg.register(rect(1, 0, 1, 1));
        let c = reg.register(rect(2, 0, 1, 1));
        let ids: Vec<_> = reg.ids().collect();
        assert_eq!(ids, vec![a, b, c]);
    }

    #[test]
    fn clear_removes_all_views() {
        let mut reg = ViewRegistry::default();
        reg.register(rect(0, 0, 10, 5));
        reg.register(rect(0, 5, 10, 5));
        reg.clear();
        assert!(reg.is_empty());
    }

    #[test]
    fn unregister_unknown_id_is_noop() {
        let mut reg = ViewRegistry::default();
        let id = reg.register(rect(0, 0, 10, 5));
        reg.unregister(id);
        // second unregister must not panic
        reg.unregister(id);
        assert!(reg.is_empty());
    }
}
