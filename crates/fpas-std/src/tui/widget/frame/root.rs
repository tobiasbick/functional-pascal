//! Atomic frame-root creation on top of the retained view registry.
//!
//! This validates frame geometry before mutating the view tree so callers never observe a partial
//! frame root when geometry is invalid.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::{ModalId, ModalStack, ViewId, ViewOptions, ViewRect, ViewRegistry};

use super::{FrameCapabilities, FrameContentSize, FrameGeometry, FrameGeometryError, FrameKind};

/// Input model for creating one framed root view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRootSpec {
    /// Window or dialog semantic kind.
    pub kind: FrameKind,
    /// Desired outer frame rectangle before desktop constraints.
    pub outer: ViewRect,
    /// Logical content size used to resolve scroll-bar visibility.
    pub content_size: FrameContentSize,
    /// Implemented geometry capabilities.
    pub capabilities: FrameCapabilities,
    /// Retained-view behavior options for the root.
    pub options: ViewOptions,
}

impl FrameRootSpec {
    /// Create a frame root spec using the current default capabilities for `kind`.
    #[must_use]
    pub fn new(kind: FrameKind, outer: ViewRect, content_size: FrameContentSize) -> Self {
        Self {
            kind,
            outer,
            content_size,
            capabilities: kind.default_capabilities(),
            options: ViewOptions::default(),
        }
    }
}

/// Registered frame root and its resolved geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRoot {
    /// Newly registered root view id.
    pub view_id: ViewId,
    /// Semantic frame kind.
    pub kind: FrameKind,
    /// Resolved frame geometry for the registered root rectangle.
    pub geometry: FrameGeometry,
}

/// Registered owned framed-dialog root and its modal id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramedDialogRoot {
    /// Registered frame root.
    pub frame: FrameRoot,
    /// Application-defined modal id pushed for this dialog.
    pub modal_id: ModalId,
}

/// Atomically register an owned framed-dialog root and push its modal frame.
///
/// The frame geometry is validated before either the view registry or modal stack is mutated.
/// Once validation succeeds, registering the view and pushing the owned modal frame are infallible
/// in-memory operations, so callers never observe a modal stack entry without its owned frame root.
pub fn register_framed_dialog_root(
    views: &mut ViewRegistry,
    modals: &mut ModalStack,
    modal_id: ModalId,
    spec: FrameRootSpec,
) -> Result<FramedDialogRoot, FrameGeometryError> {
    let frame = views.register_frame_root(spec)?;
    modals.show_dialog(modal_id, frame.view_id);

    Ok(FramedDialogRoot { frame, modal_id })
}

impl ViewRegistry {
    /// Atomically register a desktop-constrained frame root.
    ///
    /// The candidate rectangle is first constrained by the desktop metrics. Frame geometry is then
    /// validated against the constrained rectangle. The view tree is mutated only after validation
    /// succeeds.
    pub fn register_frame_root(
        &mut self,
        spec: FrameRootSpec,
    ) -> Result<FrameRoot, FrameGeometryError> {
        let outer = self.constrain_window_rect(spec.outer);
        let geometry = FrameGeometry::resolve(outer, spec.content_size, spec.capabilities)?;
        let view_id = self.register_with_options(outer, spec.options);

        Ok(FrameRoot {
            view_id,
            kind: spec.kind,
            geometry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ViewOptions;

    fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
        ViewRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn frame_kind_defaults_enable_only_implemented_scroll_geometry() {
        assert_eq!(
            FrameKind::Window.default_capabilities(),
            FrameCapabilities::scrollable()
        );
        assert_eq!(
            FrameKind::Dialog.default_capabilities(),
            FrameCapabilities::scrollable()
        );
        assert!(!FrameKind::Window.default_capabilities().closable);
        assert!(!FrameKind::Window.default_capabilities().zoomable);
    }

    #[test]
    fn register_frame_root_constrains_to_desktop_and_returns_geometry() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(2, 1, 30, 12)));
        registry.set_min_window_size(6, 6);

        let root = registry
            .register_frame_root(FrameRootSpec::new(
                FrameKind::Window,
                rect(40, 20, 10, 8),
                FrameContentSize::new(20, 20),
            ))
            .expect("valid constrained frame");

        assert_eq!(root.kind, FrameKind::Window);
        assert_eq!(root.geometry.outer, rect(22, 5, 10, 8));
        assert_eq!(registry.rect(root.view_id), Some(rect(22, 5, 10, 8)));
        assert_eq!(registry.roots(), &[root.view_id]);
        assert_eq!(root.geometry.scrollbars.vertical, Some(rect(30, 6, 1, 5)));
        assert_eq!(
            root.geometry.scrollbars.horizontal,
            Some(rect(23, 11, 7, 1))
        );
    }

    #[test]
    fn register_frame_root_uses_supplied_view_options() {
        let mut registry = ViewRegistry::default();
        let options = ViewOptions {
            selectable: true,
            tab_stop: true,
            ..ViewOptions::default()
        };

        let root = registry
            .register_frame_root(FrameRootSpec {
                kind: FrameKind::Dialog,
                outer: rect(0, 0, 8, 6),
                content_size: FrameContentSize::new(1, 1),
                capabilities: FrameCapabilities::plain(),
                options,
            })
            .expect("valid frame");

        assert_eq!(registry.options(root.view_id), Some(options));
    }

    #[test]
    fn register_frame_root_does_not_mutate_tree_on_geometry_error() {
        let mut registry = ViewRegistry::default();

        assert_eq!(
            registry.register_frame_root(FrameRootSpec::new(
                FrameKind::Window,
                rect(0, 0, 5, 6),
                FrameContentSize::new(0, 0),
            )),
            Err(FrameGeometryError {
                min_width: 6,
                min_height: 6,
                got_width: 5,
                got_height: 6,
            })
        );
        assert!(registry.is_empty());
        assert!(registry.roots().is_empty());
    }

    #[test]
    fn register_frame_root_rejects_when_desktop_is_smaller_than_frame_minimum() {
        let mut registry = ViewRegistry::default();
        assert!(registry.set_desktop_work_area(rect(0, 0, 5, 5)));

        assert_eq!(
            registry.register_frame_root(FrameRootSpec::new(
                FrameKind::Dialog,
                rect(0, 0, 10, 10),
                FrameContentSize::new(0, 0),
            )),
            Err(FrameGeometryError {
                min_width: 6,
                min_height: 6,
                got_width: 5,
                got_height: 5,
            })
        );
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn register_framed_dialog_root_validates_before_mutating_views_or_modals() {
        let mut views = ViewRegistry::default();
        let mut modals = ModalStack::default();

        assert_eq!(
            register_framed_dialog_root(
                &mut views,
                &mut modals,
                ModalId(10),
                FrameRootSpec::new(
                    FrameKind::Dialog,
                    rect(0, 0, 5, 6),
                    FrameContentSize::new(0, 0),
                ),
            ),
            Err(FrameGeometryError {
                min_width: 6,
                min_height: 6,
                got_width: 5,
                got_height: 6,
            })
        );

        assert!(views.is_empty());
        assert!(modals.is_empty());
    }

    #[test]
    fn register_framed_dialog_root_pushes_owned_modal_frame() {
        let mut views = ViewRegistry::default();
        let mut modals = ModalStack::default();

        let dialog = register_framed_dialog_root(
            &mut views,
            &mut modals,
            ModalId(42),
            FrameRootSpec::new(
                FrameKind::Dialog,
                rect(2, 3, 12, 8),
                FrameContentSize::new(20, 20),
            ),
        )
        .expect("valid framed dialog");

        assert_eq!(dialog.modal_id, ModalId(42));
        assert_eq!(dialog.frame.kind, FrameKind::Dialog);
        assert_eq!(dialog.frame.geometry.outer, rect(2, 3, 12, 8));
        assert_eq!(views.roots(), &[dialog.frame.view_id]);
        assert_eq!(modals.active_id(), Some(ModalId(42)));
        assert_eq!(modals.active_root_view(), Some(dialog.frame.view_id));
        assert_eq!(
            modals.leave_with_scope_info(),
            Some((ModalId(42), Some(dialog.frame.view_id), true, Vec::new()))
        );
    }
}
