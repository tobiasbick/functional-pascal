//! MDI window-list descriptors and direct window activation.
//!
//! Spec: `docs/pascal/std/tui/app/frames.md`

use crate::{RootActivation, ViewId, ViewRegistry};

use super::FrameKind;

/// One row in an MDI window list before the VM attaches the painted title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameWindowDescriptor {
    /// Frame root view id.
    pub id: ViewId,
    /// Window or dialog semantic kind.
    pub kind: FrameKind,
    /// Back-to-front position in the root z-order (`0` = back).
    pub z_index: usize,
    /// Whether this root is the active window root.
    pub active: bool,
}

impl ViewRegistry {
    /// Return window-kind frame roots in back-to-front z-order.
    ///
    /// Skips ids in `exclude` (for example the active modal root).
    pub fn frame_window_descriptors_excluding(
        &self,
        exclude: &[ViewId],
    ) -> Vec<FrameWindowDescriptor> {
        let active = self.active_root().or_else(|| self.roots().last().copied());
        self.roots()
            .iter()
            .enumerate()
            .filter_map(|(z_index, root)| {
                if exclude.contains(root) {
                    return None;
                }
                let state = self.frame_roots.get(root)?;
                if state.kind != FrameKind::Window {
                    return None;
                }
                Some(FrameWindowDescriptor {
                    id: *root,
                    kind: state.kind,
                    z_index,
                    active: active == Some(*root),
                })
            })
            .collect()
    }

    /// Raise and focus the frame root containing `id`.
    ///
    /// Returns `None` for unknown ids and non-frame roots.
    pub fn activate_frame_window(&mut self, id: ViewId) -> Option<RootActivation> {
        let root = self.root_of(id)?;
        if !self.frame_roots.contains_key(&root) {
            return None;
        }
        self.activate_root(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrameCapabilities, FrameContentSize, FrameRootSpec, ViewRect, ViewRegistry};

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    fn window_spec(outer: ViewRect) -> FrameRootSpec {
        FrameRootSpec {
            kind: FrameKind::Window,
            outer,
            content_size: FrameContentSize::new(0, 0),
            capabilities: FrameCapabilities {
                movable: true,
                resizable: true,
                zoomable: false,
                closable: false,
                scrollable: false,
            },
            options: Default::default(),
        }
    }

    #[test]
    fn descriptors_list_window_roots_in_z_order() {
        let mut registry = ViewRegistry::default();
        let back = registry
            .register_frame_root(window_spec(rect(2, 2, 20, 8)))
            .expect("back")
            .view_id;
        let front = registry
            .register_frame_root(window_spec(rect(30, 4, 20, 8)))
            .expect("front")
            .view_id;
        registry
            .register_frame_root(FrameRootSpec {
                kind: FrameKind::Dialog,
                outer: rect(10, 6, 18, 6),
                content_size: FrameContentSize::new(0, 0),
                capabilities: FrameCapabilities::plain(),
                options: Default::default(),
            })
            .expect("dialog");

        let list = registry.frame_window_descriptors_excluding(&[]);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, back);
        assert_eq!(list[0].z_index, 0);
        assert_eq!(list[1].id, front);
        assert_eq!(list[1].z_index, 1);
    }

    #[test]
    fn activate_frame_window_raises_requested_root() {
        let mut registry = ViewRegistry::default();
        let back = registry
            .register_frame_root(window_spec(rect(2, 2, 20, 8)))
            .expect("back")
            .view_id;
        let _front = registry
            .register_frame_root(window_spec(rect(30, 4, 20, 8)))
            .expect("front")
            .view_id;

        let activation = registry.activate_frame_window(back).expect("activate");
        assert_eq!(activation.root, back);
        assert!(activation.raised);
        assert_eq!(registry.roots().last().copied(), Some(back));
    }
}
