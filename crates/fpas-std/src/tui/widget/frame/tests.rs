//! Phase 2 frame acceptance scenarios for the retained desktop.
//!
//! These tests assemble real frame roots on the retained [`ViewRegistry`] and verify the
//! cross-cutting desktop behavior the window/dialog plan requires before frame painting: overlap
//! z-order, occlusion repair when a covering frame is removed, child clipping, click-driven focus
//! activation between windows, and nested frames.
//!
//! Plan: `docs/future/windows-dialogs/README.md`
//! Review: `docs/future/windows-dialogs/TUI-CODE-REVIEW.md`

use crate::{
    FrameContentSize, FrameKind, FrameRootSpec, ViewId, ViewRect, ViewRegistry, WindowPalette,
};

fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
    ViewRect {
        x,
        y,
        width,
        height,
    }
}

fn registry_with_desktop() -> ViewRegistry {
    let mut registry = ViewRegistry::default();
    assert!(registry.set_desktop_work_area(rect(0, 0, 80, 25)));
    registry
}

fn window_spec(outer: ViewRect) -> FrameRootSpec {
    FrameRootSpec::new(FrameKind::Window, outer, FrameContentSize::new(0, 0))
}

fn focusable_leaf(registry: &mut ViewRegistry, parent: ViewId, leaf: ViewRect) -> ViewId {
    let id = registry.register(leaf);
    assert!(registry.set_parent(id, Some(parent)));
    assert!(registry.push_child(id));
    id
}

#[test]
fn overlapping_frames_resolve_topmost_by_z_order() {
    let mut registry = registry_with_desktop();
    let back = registry
        .register_frame_root(window_spec(rect(10, 2, 20, 8)))
        .expect("valid back frame");
    let front = registry
        .register_frame_root(window_spec(rect(16, 4, 20, 8)))
        .expect("valid front frame");

    // (20, 6) is inside both frame rectangles; the later root wins the z-order.
    assert_eq!(registry.topmost_view_at(20, 6, None), Some(front.view_id));

    assert!(registry.raise(back.view_id));
    assert_eq!(registry.topmost_view_at(20, 6, None), Some(back.view_id));
}

#[test]
fn closing_front_frame_reexposes_occluded_frame() {
    let mut registry = registry_with_desktop();
    let back = registry
        .register_frame_root(window_spec(rect(10, 2, 20, 8)))
        .expect("valid back frame");
    let front = registry
        .register_frame_root(window_spec(rect(16, 4, 20, 8)))
        .expect("valid front frame");

    let covered = registry
        .rect(front.view_id)
        .and_then(|front_rect| front_rect.intersection(registry.rect(back.view_id).unwrap()))
        .expect("frames overlap");
    assert_eq!(registry.topmost_view_at(20, 6, None), Some(front.view_id));

    registry.unregister(front.view_id);

    // The covering frame is gone, so the occluded frame paints and hit-tests the shared cells again.
    assert_eq!(registry.paint_order(), vec![back.view_id]);
    assert!(covered.intersects(registry.rect(back.view_id).unwrap()));
    assert_eq!(registry.topmost_view_at(20, 6, None), Some(back.view_id));
}

#[test]
fn frame_child_is_clipped_to_frame_bounds() {
    let mut registry = registry_with_desktop();
    let frame = registry
        .register_frame_root(window_spec(rect(10, 2, 10, 8)))
        .expect("valid frame");

    let view = frame.geometry.view;
    let child = registry.register(rect(view.x, view.y, view.width + 30, view.height + 30));
    assert!(registry.set_parent(child, Some(frame.view_id)));

    // The frame outer rectangle is x:10..20, y:2..10; the oversized child clips to it.
    let resolved = registry.resolved(child).expect("child resolves");
    assert_eq!(resolved.clip, Some(rect(11, 3, 9, 7)));
    assert!(resolved.state.exposed);
}

#[test]
fn clicking_occluded_window_raises_and_moves_focus() {
    let mut registry = registry_with_desktop();
    let back = registry
        .register_frame_root(window_spec(rect(2, 2, 20, 8)))
        .expect("valid back frame");
    let front = registry
        .register_frame_root(window_spec(rect(30, 4, 20, 8)))
        .expect("valid front frame");
    let back_leaf = focusable_leaf(&mut registry, back.view_id, rect(4, 4, 6, 1));
    let front_leaf = focusable_leaf(&mut registry, front.view_id, rect(32, 6, 6, 1));

    assert_eq!(registry.focus_view(front_leaf), (true, false));
    assert_eq!(registry.active_root(), Some(front.view_id));
    assert_eq!(
        registry.root_palette(back.view_id),
        Some(WindowPalette::Inactive)
    );

    let hit = registry
        .topmost_view_at(4, 4, None)
        .expect("click hits the back window");
    let activation = registry.activate_root(hit).expect("known view");

    assert!(activation.raised);
    assert!(activation.focus_changed);
    assert_eq!(registry.roots(), &[front.view_id, back.view_id]);
    assert_eq!(registry.focused_id(), Some(back_leaf));
    assert_eq!(registry.active_root(), Some(back.view_id));
    assert_eq!(
        registry.root_palette(back.view_id),
        Some(WindowPalette::Active)
    );
    assert_eq!(
        registry.root_palette(front.view_id),
        Some(WindowPalette::Inactive)
    );
}

#[test]
fn nested_frame_clips_to_parent_and_activates_through_root() {
    let mut registry = registry_with_desktop();
    let window = registry
        .register_frame_root(window_spec(rect(5, 2, 20, 10)))
        .expect("valid window frame");
    let dialog = registry
        .register_frame_root(FrameRootSpec::new(
            FrameKind::Dialog,
            rect(18, 4, 12, 8),
            FrameContentSize::new(0, 0),
        ))
        .expect("valid dialog frame");

    // Nest the dialog frame inside the window frame; it keeps its absolute rectangle.
    assert!(registry.set_parent(dialog.view_id, Some(window.view_id)));
    let leaf = focusable_leaf(&mut registry, dialog.view_id, rect(20, 6, 4, 1));

    assert_eq!(registry.root_of(dialog.view_id), Some(window.view_id));
    assert_eq!(registry.root_of(leaf), Some(window.view_id));

    // The dialog overflows the window on the right and is clipped to the window bounds.
    let dialog_clip = registry
        .resolved(dialog.view_id)
        .expect("dialog resolves")
        .clip;
    assert_eq!(dialog_clip, Some(rect(18, 4, 7, 8)));

    // Focusing a deeply nested leaf activates the outer window root.
    assert_eq!(registry.focus_view(leaf), (true, false));
    assert_eq!(registry.active_root(), Some(window.view_id));
    assert_eq!(
        registry.root_palette(window.view_id),
        Some(WindowPalette::Active)
    );
}
