//! Host-widget target selection for TUI keyboard and pointer routing.
//!
//! **Documentation:** `docs/pascal/std/tui/app/README.md` (from the repository root).

use std::collections::HashMap;

use fpas_std::{UiMouse, ViewId, ViewRect, ViewRegistry, ViewWidget};

pub(super) fn topmost_menu_bar(
    views: &ViewRegistry,
    widgets: &HashMap<ViewId, ViewWidget>,
    scope: Option<&[ViewId]>,
) -> Option<(ViewId, ViewRect)> {
    views
        .resolved_paint_order()
        .into_iter()
        .rev()
        .filter(|view| is_in_scope(view.id, scope))
        .find_map(|view| {
            let widget = widgets.get(&view.id)?;
            matches!(widget, ViewWidget::MenuBar(_)).then_some((view.id, view.rect))
        })
}

pub(super) fn widget_mouse_hit(
    views: &ViewRegistry,
    widgets: &HashMap<ViewId, ViewWidget>,
    mouse: UiMouse,
    scope: Option<&[ViewId]>,
) -> Option<(ViewId, ViewRect)> {
    if let Some(captured) = views.captured_pointer()
        && is_in_scope(captured, scope)
        && let Some(view) = views.resolved(captured)
        && view.state.enabled
        && widgets.contains_key(&captured)
    {
        return Some((captured, view.rect));
    }

    let order = views.resolved_paint_order();

    let menu_hit = order
        .iter()
        .rev()
        .filter(|view| is_in_scope(view.id, scope) && view.state.enabled)
        .find_map(|view| {
            let widget = widgets.get(&view.id)?;
            let ViewWidget::MenuBar(menu) = widget else {
                return None;
            };
            menu.contains_point(view.rect, mouse.x, mouse.y)
                .then_some((view.id, view.rect))
        });
    if menu_hit.is_some() {
        return menu_hit;
    }

    order
        .into_iter()
        .rev()
        .filter(|view| is_in_scope(view.id, scope) && view.state.enabled)
        .find_map(|view| {
            widgets.contains_key(&view.id).then_some(())?;
            view.clip
                .is_some_and(|clip| clip.contains_console_mouse(mouse.x, mouse.y))
                .then_some((view.id, view.rect))
        })
}

fn is_in_scope(view_id: ViewId, scope: Option<&[ViewId]>) -> bool {
    scope.is_none_or(|view_ids| view_ids.contains(&view_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpas_std::{MenuBarStyle, MenuBarWidget, UiModifiers};

    fn rect() -> ViewRect {
        ViewRect {
            x: 0,
            y: 0,
            width: 20,
            height: 1,
        }
    }

    fn mouse() -> UiMouse {
        UiMouse::new(0, 0, 1, 1, UiModifiers::default())
    }

    fn menu_widget() -> ViewWidget {
        ViewWidget::MenuBar(MenuBarWidget::new(Vec::new(), MenuBarStyle::default()))
    }

    #[test]
    fn mouse_hit_skips_topmost_view_without_widget() {
        let mut views = ViewRegistry::default();
        let menu = views.register(rect());
        let _plain = views.register(rect());
        let widgets = HashMap::from([(menu, menu_widget())]);

        let hit = widget_mouse_hit(&views, &widgets, mouse(), None);

        assert_eq!(hit.map(|(view_id, _)| view_id), Some(menu));
    }

    #[test]
    fn menu_target_respects_modal_scope() {
        let mut views = ViewRegistry::default();
        let menu = views.register(rect());
        let modal = views.register(rect());
        let widgets = HashMap::from([(menu, menu_widget())]);

        assert!(topmost_menu_bar(&views, &widgets, Some(&[modal])).is_none());
        assert!(widget_mouse_hit(&views, &widgets, mouse(), Some(&[modal])).is_none());
    }
}
