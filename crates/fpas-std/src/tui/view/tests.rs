use super::*;

fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
    ViewRect {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn register_returns_distinct_ids() {
    let mut registry = ViewRegistry::default();
    let a = registry.register(rect(0, 0, 10, 5));
    let b = registry.register(rect(10, 0, 20, 5));

    assert_ne!(a, b);
    assert_eq!(registry.len(), 2);
}

#[test]
fn roots_lists_root_views_in_registration_order() {
    let mut registry = ViewRegistry::default();
    let first = registry.register(rect(0, 0, 10, 1));
    let second = registry.register(rect(0, 1, 10, 9));

    assert_eq!(registry.roots(), &[first, second]);
}

#[test]
fn parent_and_children_track_reparenting() {
    let mut registry = ViewRegistry::default();
    let parent = registry.register(rect(0, 0, 40, 20));
    let first = registry.register(rect(1, 1, 10, 5));
    let second = registry.register(rect(12, 1, 10, 5));

    assert!(registry.set_parent(first, Some(parent)));
    assert!(registry.set_parent(second, Some(parent)));

    assert_eq!(registry.parent(first), Some(parent));
    assert_eq!(registry.parent(second), Some(parent));
    assert_eq!(registry.parent(parent), None);
    assert_eq!(registry.children(parent), &[first, second]);
}

#[test]
fn register_wraps_id_allocator_without_reusing_live_ids() {
    let mut registry = ViewRegistry {
        next_id: u32::MAX,
        ..ViewRegistry::default()
    };

    let max_id = registry.register(rect(0, 0, 1, 1));
    let wrapped_id = registry.register(rect(1, 0, 1, 1));

    assert_eq!(max_id, ViewId::from_raw(u32::MAX));
    assert_eq!(wrapped_id, ViewId::from_raw(0));
}

#[test]
fn child_rect_tracks_parent_layout() {
    let mut registry = ViewRegistry::default();
    let parent = registry.register(rect(10, 5, 20, 10));
    let child = registry.register(rect(2, 3, 4, 2));

    assert!(registry.set_parent(child, Some(parent)));
    registry.set_rect(child, rect(1, 1, 4, 2));
    assert_eq!(registry.rect(child), Some(rect(11, 6, 4, 2)));

    registry.set_rect(parent, rect(20, 10, 20, 10));
    assert_eq!(registry.rect(child), Some(rect(21, 11, 4, 2)));
}

#[test]
fn rect_contains_point_handles_overflow_and_negative_size() {
    assert!(rect(i64::MAX - 1, i64::MAX - 1, 10, 10).contains_point(i64::MAX - 1, i64::MAX - 1));
    assert!(!rect(0, 0, -1, 10).contains_point(0, 0));
}

#[test]
fn rect_intersection_returns_shared_cells() {
    assert_eq!(
        rect(2, 3, 5, 4).intersection(rect(5, 1, 4, 5)),
        Some(rect(5, 3, 2, 3))
    );
    assert_eq!(rect(0, 0, 2, 2).intersection(rect(2, 0, 2, 2)), None);
}

#[test]
fn rect_union_returns_bounding_rectangle() {
    assert_eq!(rect(2, 3, 5, 4).union(rect(5, 1, 4, 5)), rect(2, 1, 7, 6));
}

#[test]
fn reparent_preserves_absolute_rect() {
    let mut registry = ViewRegistry::default();
    let first_parent = registry.register(rect(10, 5, 20, 10));
    let second_parent = registry.register(rect(40, 20, 20, 10));
    let child = registry.register(rect(14, 9, 4, 2));

    assert!(registry.set_parent(child, Some(first_parent)));
    assert_eq!(registry.rect(child), Some(rect(14, 9, 4, 2)));

    assert!(registry.set_parent(child, Some(second_parent)));
    assert_eq!(registry.rect(child), Some(rect(14, 9, 4, 2)));
}

#[test]
fn reparent_rejects_cycles() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 10, 10));
    let child = registry.register(rect(1, 1, 4, 4));
    let grandchild = registry.register(rect(1, 1, 2, 2));

    assert!(registry.set_parent(child, Some(root)));
    assert!(registry.set_parent(grandchild, Some(child)));
    assert!(!registry.set_parent(root, Some(grandchild)));
}

#[test]
fn unregister_removes_subtree() {
    let mut registry = ViewRegistry::default();
    let parent = registry.register(rect(0, 0, 10, 5));
    let child = registry.register(rect(1, 1, 4, 2));
    registry.push_child(parent);
    registry.push_child(child);
    assert!(registry.set_parent(child, Some(parent)));

    registry.unregister(parent);

    assert!(registry.is_empty());
    assert_eq!(registry.focused_id(), None);
    assert_eq!(registry.rect(child), None);
}

#[test]
fn unregister_rebuilds_view_lookup_index() {
    let mut registry = ViewRegistry::default();
    let first = registry.register(rect(0, 0, 10, 5));
    let removed = registry.register(rect(10, 0, 10, 5));
    let remaining = registry.register(rect(20, 0, 10, 5));

    registry.unregister(removed);
    let added = registry.register(rect(30, 0, 10, 5));
    registry.set_rect(remaining, rect(22, 2, 8, 4));

    assert_eq!(registry.rect(first), Some(rect(0, 0, 10, 5)));
    assert_eq!(registry.rect(removed), None);
    assert_eq!(registry.rect(remaining), Some(rect(22, 2, 8, 4)));
    assert_eq!(registry.rect(added), Some(rect(30, 0, 10, 5)));
    assert_eq!(
        registry.ids().collect::<Vec<_>>(),
        vec![first, remaining, added]
    );
}

#[test]
fn paint_order_follows_tree_and_raise() {
    let mut registry = ViewRegistry::default();
    let background = registry.register(rect(0, 0, 80, 25));
    let window = registry.register(rect(10, 5, 20, 10));
    let button = registry.register(rect(1, 1, 6, 1));

    assert!(registry.set_parent(button, Some(window)));
    assert_eq!(registry.paint_order(), vec![background, window, button]);

    assert!(registry.raise(background));
    assert_eq!(registry.paint_order(), vec![window, button, background]);
}

#[test]
fn topmost_view_at_uses_current_z_order() {
    let mut registry = ViewRegistry::default();
    let back = registry.register(rect(0, 0, 10, 10));
    let front = registry.register(rect(0, 0, 10, 10));

    assert_eq!(registry.topmost_view_at(2, 2, None), Some(front));
    assert!(registry.raise(back));
    assert_eq!(registry.topmost_view_at(2, 2, None), Some(back));
}

#[test]
fn topmost_view_at_can_be_scoped_to_subtree() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 30, 10));
    let sibling = registry.register(rect(0, 0, 30, 10));
    let child = registry.register(rect(1, 1, 10, 3));
    assert!(registry.set_parent(child, Some(root)));

    assert_eq!(registry.topmost_view_at(2, 2, None), Some(sibling));
    assert_eq!(
        registry.topmost_view_at(2, 2, Some(&registry.subtree_ids(root))),
        Some(child)
    );
}

#[test]
fn focus_first_in_scope_targets_first_matching_focus_child() {
    let mut registry = ViewRegistry::default();
    let a = registry.register(rect(0, 0, 1, 1));
    let b = registry.register(rect(1, 0, 1, 1));
    let c = registry.register(rect(2, 0, 1, 1));
    registry.push_child(a);
    registry.push_child(b);
    registry.push_child(c);

    let (changed, had_previous) = registry.focus_first_in_scope(&[b, c]);
    assert!(changed);
    assert!(!had_previous);
    assert_eq!(registry.focused_id(), Some(b));
}

#[test]
fn push_child_rejects_unknown_view_id() {
    let mut registry = ViewRegistry::default();

    assert!(!registry.push_child(ViewId::from_raw(42)));
    assert!(!registry.has_focusable_children());
}

#[test]
fn focus_next_two_children_wraps() {
    let mut registry = ViewRegistry::default();
    let a = registry.register(rect(0, 0, 10, 5));
    let b = registry.register(rect(0, 5, 10, 5));
    registry.push_child(a);
    registry.push_child(b);

    let (changed_a, had_a) = registry.focus_next();
    assert!(changed_a);
    assert!(!had_a);
    assert_eq!(registry.focused_id(), Some(a));

    let (changed_b, had_b) = registry.focus_next();
    assert!(changed_b);
    assert!(had_b);
    assert_eq!(registry.focused_id(), Some(b));

    let (changed_wrap, had_wrap) = registry.focus_next();
    assert!(changed_wrap);
    assert!(had_wrap);
    assert_eq!(registry.focused_id(), Some(a));
}

#[test]
fn remove_child_adjusts_focus() {
    let mut registry = ViewRegistry::default();
    let a = registry.register(rect(0, 0, 1, 1));
    let b = registry.register(rect(1, 0, 1, 1));
    let c = registry.register(rect(2, 0, 1, 1));
    registry.push_child(a);
    registry.push_child(b);
    registry.push_child(c);

    let _ = registry.focus_next();
    let _ = registry.focus_next();
    assert_eq!(registry.focused_id(), Some(b));

    registry.remove_child(b);
    assert_eq!(registry.focused_id(), Some(a));
}

#[test]
fn scoped_focus_wraps_inside_scope_only() {
    let mut registry = ViewRegistry::default();
    let a = registry.register(rect(0, 0, 1, 1));
    let b = registry.register(rect(1, 0, 1, 1));
    let c = registry.register(rect(2, 0, 1, 1));
    registry.push_child(a);
    registry.push_child(b);
    registry.push_child(c);

    let (changed_first, had_previous_first) = registry.focus_first_in_scope(&[b, c]);
    assert!(changed_first);
    assert!(!had_previous_first);
    assert_eq!(registry.focused_id(), Some(b));

    let (changed_next, had_previous_next) = registry.focus_next_in_scope(&[b, c]);
    assert!(changed_next);
    assert!(had_previous_next);
    assert_eq!(registry.focused_id(), Some(c));

    let (changed_wrap, had_previous_wrap) = registry.focus_next_in_scope(&[b, c]);
    assert!(changed_wrap);
    assert!(had_previous_wrap);
    assert_eq!(registry.focused_id(), Some(b));
}

#[test]
fn ancestors_inclusive_returns_leaf_to_root() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 10, 5));
    let child = registry.register(rect(1, 1, 4, 2));
    let leaf = registry.register(rect(1, 0, 2, 1));

    assert!(registry.set_parent(child, Some(root)));
    assert!(registry.set_parent(leaf, Some(child)));

    assert_eq!(registry.ancestors_inclusive(leaf), vec![leaf, child, root]);
}

#[test]
fn resolved_view_clips_nested_child_to_parent() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(10, 5, 8, 4));
    let child = registry.register(rect(6, 2, 6, 4));
    assert!(registry.set_parent(child, Some(root)));
    registry.set_rect(child, rect(6, 2, 6, 4));

    let resolved = registry.resolved(child).expect("child should resolve");
    assert_eq!(resolved.rect, rect(16, 7, 6, 4));
    assert_eq!(resolved.clip, Some(rect(16, 7, 2, 2)));
}

#[test]
fn resolved_subtree_keeps_snapshot_order_and_hidden_rects() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(10, 5, 8, 4));
    let child = registry.register(rect(6, 2, 6, 4));
    let leaf = registry.register(rect(1, 1, 2, 1));
    assert!(registry.set_parent(child, Some(root)));
    assert!(registry.set_parent(leaf, Some(child)));
    registry.set_rect(child, rect(6, 2, 6, 4));
    registry.set_rect(leaf, rect(1, 1, 2, 1));
    assert!(registry.set_visible(child, false));

    let snapshot = registry.resolved_subtree(root);
    assert_eq!(
        snapshot.iter().map(|view| view.id).collect::<Vec<_>>(),
        vec![root, child, leaf]
    );
    assert_eq!(snapshot[0].rect, rect(10, 5, 8, 4));
    assert_eq!(snapshot[1].rect, rect(16, 7, 6, 4));
    assert_eq!(snapshot[2].rect, rect(17, 8, 2, 1));
    assert_eq!(snapshot[0].content_origin, (10, 5));
    assert_eq!(snapshot[1].content_origin, (16, 7));
    assert_eq!(snapshot[2].content_origin, (17, 8));
    assert_eq!(snapshot[1].clip, None);
    assert!(!snapshot[1].state.exposed);
}

#[test]
fn hidden_parent_hides_descendants_from_paint_and_hit_test() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 10, 5));
    let child = registry.register(rect(1, 1, 4, 2));
    assert!(registry.set_parent(child, Some(root)));
    assert!(registry.set_visible(root, false));

    assert!(registry.resolved_paint_order().is_empty());
    assert_eq!(registry.topmost_view_at(2, 2, None), None);
    assert!(!registry.state(child).expect("child state").exposed);
}

#[test]
fn focus_path_tracks_group_current_child() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 20, 10));
    let group = registry.register(rect(1, 1, 10, 5));
    let leaf = registry.register(rect(1, 1, 4, 1));
    assert!(registry.set_parent(group, Some(root)));
    assert!(registry.set_parent(leaf, Some(group)));
    assert!(registry.push_child(leaf));

    assert_eq!(registry.focus_view(leaf), (true, false));
    assert_eq!(registry.current_child(root), Some(group));
    assert_eq!(registry.current_child(group), Some(leaf));
    assert!(registry.state(root).expect("root state").active);
    assert!(registry.state(leaf).expect("leaf state").focused);
}

#[test]
fn disabled_or_hidden_views_are_skipped_during_focus_traversal() {
    let mut registry = ViewRegistry::default();
    let disabled = registry.register(rect(0, 0, 2, 1));
    let hidden = registry.register(rect(2, 0, 2, 1));
    let enabled = registry.register(rect(4, 0, 2, 1));
    for id in [disabled, hidden, enabled] {
        assert!(registry.push_child(id));
    }
    assert!(registry.set_enabled(disabled, false));
    assert!(registry.set_visible(hidden, false));

    assert_eq!(registry.focus_next(), (true, false));
    assert_eq!(registry.focused_id(), Some(enabled));
}

#[test]
fn routed_pointer_event_builds_capture_and_bubble_paths() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 20, 10));
    let child = registry.register(rect(1, 1, 10, 5));
    let leaf = registry.register(rect(1, 1, 4, 1));
    assert!(registry.set_parent(child, Some(root)));
    assert!(registry.set_parent(leaf, Some(child)));
    registry.set_rect(child, rect(1, 1, 10, 5));
    registry.set_rect(leaf, rect(1, 1, 4, 1));
    for id in [root, child] {
        let mut options = registry.options(id).expect("view options");
        options.pre_process = true;
        options.post_process = true;
        assert!(registry.set_options(id, options));
    }

    let event = RoutedEvent::Mouse(crate::UiMouse::new(1, 1, 3, 3, Default::default()));
    let route = registry.route_event(event, None);
    assert_eq!(route.target, Some(leaf));
    assert_eq!(route.capture, vec![root, child]);
    assert_eq!(route.bubble, vec![child, root]);
}

#[test]
fn pointer_capture_routes_outside_bounds_and_clears_on_removal() {
    let mut registry = ViewRegistry::default();
    let view = registry.register(rect(1, 1, 4, 2));
    assert!(registry.capture_pointer(view));

    assert_eq!(registry.pointer_target(80, 25, None), Some(view));
    registry.unregister(view);
    assert_eq!(registry.captured_pointer(), None);
    assert_eq!(registry.pointer_target(80, 25, None), None);
}

#[test]
fn root_of_returns_root_ancestor_and_self_for_root() {
    let mut registry = ViewRegistry::default();
    let root = registry.register(rect(0, 0, 20, 10));
    let group = registry.register(rect(1, 1, 10, 5));
    let leaf = registry.register(rect(1, 1, 4, 1));
    assert!(registry.set_parent(group, Some(root)));
    assert!(registry.set_parent(leaf, Some(group)));

    assert_eq!(registry.root_of(leaf), Some(root));
    assert_eq!(registry.root_of(root), Some(root));
    assert_eq!(registry.root_of(ViewId::from_raw(99)), None);
}

#[test]
fn active_root_follows_focused_leaf() {
    let mut registry = ViewRegistry::default();
    let window = registry.register(rect(0, 0, 20, 10));
    let leaf = registry.register(rect(1, 1, 4, 1));
    assert!(registry.set_parent(leaf, Some(window)));
    assert!(registry.push_child(leaf));

    assert_eq!(registry.active_root(), None);
    assert_eq!(registry.focus_view(leaf), (true, false));
    assert_eq!(registry.active_root(), Some(window));
}

#[test]
fn activate_root_raises_window_to_front_and_focuses_first_child() {
    let mut registry = ViewRegistry::default();
    let back = registry.register(rect(0, 0, 40, 20));
    let front = registry.register(rect(5, 5, 20, 10));
    let back_button = registry.register(rect(1, 1, 6, 1));
    assert!(registry.set_parent(back_button, Some(back)));
    assert!(registry.push_child(back_button));

    assert_eq!(registry.roots(), &[back, front]);

    let activation = registry.activate_root(back_button).expect("known view");
    assert_eq!(activation.root, back);
    assert!(activation.raised);
    assert!(activation.focus_changed);
    assert!(!activation.had_previous_focus);

    assert_eq!(registry.roots(), &[front, back]);
    assert_eq!(registry.focused_id(), Some(back_button));
    assert_eq!(registry.active_root(), Some(back));
}

#[test]
fn activate_root_keeps_focus_already_inside_target() {
    let mut registry = ViewRegistry::default();
    let window = registry.register(rect(0, 0, 20, 10));
    let first = registry.register(rect(1, 1, 4, 1));
    let second = registry.register(rect(1, 3, 4, 1));
    assert!(registry.set_parent(first, Some(window)));
    assert!(registry.set_parent(second, Some(window)));
    assert!(registry.push_child(first));
    assert!(registry.push_child(second));
    assert_eq!(registry.focus_view(second), (true, false));

    let activation = registry.activate_root(window).expect("known view");
    assert!(!activation.raised);
    assert!(!activation.focus_changed);
    assert!(activation.had_previous_focus);
    assert_eq!(registry.focused_id(), Some(second));
}

#[test]
fn activate_root_without_focusable_children_still_raises() {
    let mut registry = ViewRegistry::default();
    let back = registry.register(rect(0, 0, 40, 20));
    let front = registry.register(rect(5, 5, 20, 10));

    let activation = registry.activate_root(back).expect("known view");
    assert!(activation.raised);
    assert!(!activation.focus_changed);
    assert_eq!(registry.roots(), &[front, back]);
    assert_eq!(registry.focused_id(), None);
}

#[test]
fn activate_root_rejects_unknown_view() {
    let mut registry = ViewRegistry::default();
    assert_eq!(registry.activate_root(ViewId::from_raw(7)), None);
}

#[test]
fn desktop_work_area_rejects_empty_rectangles() {
    let mut registry = ViewRegistry::default();

    assert!(!registry.set_desktop_work_area(rect(0, 0, 0, 10)));
    assert_eq!(registry.desktop_metrics().work_area, None);

    assert!(registry.set_desktop_work_area(rect(1, 1, 20, 10)));
    assert_eq!(
        registry.desktop_metrics().work_area,
        Some(rect(1, 1, 20, 10))
    );
    registry.clear_desktop_work_area();
    assert_eq!(registry.desktop_metrics().work_area, None);
}

#[test]
fn constrain_window_rect_applies_minimum_size_and_desktop_bounds() {
    let mut registry = ViewRegistry::default();
    assert!(registry.set_desktop_work_area(rect(2, 1, 30, 12)));
    registry.set_min_window_size(8, 4);

    assert_eq!(
        registry.constrain_window_rect(rect(-10, -5, 3, 2)),
        rect(2, 1, 8, 4)
    );
    assert_eq!(
        registry.constrain_window_rect(rect(40, 20, 12, 6)),
        rect(20, 7, 12, 6)
    );
    assert_eq!(
        registry.constrain_window_rect(rect(4, 3, 100, 100)),
        rect(2, 1, 30, 12)
    );
}

#[test]
fn set_root_rect_constrained_updates_containing_root_only() {
    let mut registry = ViewRegistry::default();
    assert!(registry.set_desktop_work_area(rect(0, 0, 20, 10)));
    registry.set_min_window_size(6, 3);
    let root = registry.register(rect(0, 0, 10, 5));
    let child = registry.register(rect(1, 1, 4, 1));
    assert!(registry.set_parent(child, Some(root)));

    assert_eq!(
        registry.set_root_rect_constrained(child, rect(18, 9, 2, 2)),
        Some(rect(14, 7, 6, 3))
    );
    assert_eq!(registry.rect(root), Some(rect(14, 7, 6, 3)));
    assert_eq!(registry.rect(child), Some(rect(15, 8, 4, 1)));
    assert_eq!(
        registry.set_root_rect_constrained(ViewId::from_raw(99), rect(0, 0, 1, 1)),
        None
    );
}

#[test]
fn root_palette_reports_active_and_inactive_roots() {
    let mut registry = ViewRegistry::default();
    let first = registry.register(rect(0, 0, 20, 10));
    let second = registry.register(rect(5, 5, 20, 10));
    let first_leaf = registry.register(rect(1, 1, 4, 1));
    assert!(registry.set_parent(first_leaf, Some(first)));
    assert!(registry.push_child(first_leaf));

    assert_eq!(registry.root_palette(first), Some(WindowPalette::Inactive));
    assert_eq!(registry.root_palette(second), Some(WindowPalette::Inactive));

    assert_eq!(registry.focus_view(first_leaf), (true, false));
    assert_eq!(
        registry.root_palette(first_leaf),
        Some(WindowPalette::Active)
    );
    assert_eq!(registry.root_palette(second), Some(WindowPalette::Inactive));
    assert_eq!(registry.root_palette(ViewId::from_raw(99)), None);
}

#[test]
fn root_shadow_is_disabled_by_default_and_clips_to_work_area() {
    let mut registry = ViewRegistry::default();
    assert!(registry.set_desktop_work_area(rect(0, 0, 10, 5)));
    let root = registry.register(rect(2, 1, 4, 3));

    assert_eq!(registry.root_shadow(root), None);
    registry.set_window_shadow_enabled(true);

    assert_eq!(
        registry.root_shadow(root),
        Some(WindowShadow {
            right: Some(rect(6, 2, 1, 3)),
            bottom: Some(rect(3, 4, 4, 1)),
        })
    );
}

#[test]
fn shadow_geometry_can_be_fully_clipped() {
    let mut registry = ViewRegistry::default();
    assert!(registry.set_desktop_work_area(rect(0, 0, 6, 4)));
    registry.set_window_shadow_enabled(true);
    let root = registry.register(rect(0, 0, 6, 4));

    let shadow = registry.root_shadow(root).expect("shadow enabled");
    assert!(shadow.is_empty());
}
