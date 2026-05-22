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
