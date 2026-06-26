//! Route input to native menu-bar widgets.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::shared::TuiState;
use fpas_bytecode::SourceLocation;
use fpas_std::{CommandEvent, MenuBarMouseResult, ProcessOutcome, ViewId, ViewRect, ViewWidget};

use super::super::widget_target;

impl Worker {
    /// Routes a mouse event to host widgets before Pascal `OnMouse` handlers run.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_widget_mouse(
        &mut self,
        mouse: fpas_std::UiMouse,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::widget_mouse_hit(&tui.views, &tui.view_widgets, mouse, modal_scope)
        };

        let Some((view_id, rect)) = hit else {
            return Ok(None);
        };

        let result = self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&view_id)?;
            let ViewWidget::MenuBar(menu) = &mut widget else {
                tui.view_widgets.insert(view_id, widget);
                return Some(MenuBarMouseResult::Ignored);
            };
            let before = menu.damage_rects(rect);
            let result = menu.handle_mouse(rect, mouse);
            let after = menu.damage_rects(rect);
            tui.view_widgets.insert(view_id, widget);
            if result != MenuBarMouseResult::Ignored {
                let mut regions = before;
                regions.extend(after);
                Self::request_unique_redraws(tui, &regions, line);
            }
            Some(result)
        });

        let dispatch_tag = match result.unwrap_or(MenuBarMouseResult::Ignored) {
            MenuBarMouseResult::Ignored => return Ok(None),
            MenuBarMouseResult::HoverChanged => ProcessOutcome::Pointer { handled: true },
            MenuBarMouseResult::Command(command_id) => {
                let enabled = self.with_tui(|tui| tui.commands.is_enabled(command_id));
                if !enabled {
                    return Ok(Some(ProcessOutcome::Pointer { handled: true }));
                }
                return self
                    .dispatch_tui_command(
                        CommandEvent::application(command_id, Some(view_id)),
                        line,
                    )
                    .map(Some);
            }
        };

        Ok(Some(dispatch_tag))
    }

    /// Routes keyboard shortcuts to host menu bar widgets before global command bindings.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_widget_key(
        &mut self,
        key: fpas_std::ConsoleKeyEvent,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::topmost_menu_bar(&tui.views, &tui.view_widgets, modal_scope)
        };

        let Some((view_id, rect)) = hit else {
            return Ok(None);
        };
        let result = self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&view_id)?;
            let ViewWidget::MenuBar(menu) = &mut widget else {
                tui.view_widgets.insert(view_id, widget);
                return Some(MenuBarMouseResult::Ignored);
            };
            let before = menu.damage_rects(rect);
            let result = menu.handle_key(&key);
            let after = menu.damage_rects(rect);
            tui.view_widgets.insert(view_id, widget);
            if result != MenuBarMouseResult::Ignored {
                let mut regions = before;
                regions.extend(after);
                Self::request_unique_redraws(tui, &regions, line);
            }
            Some(result)
        });
        let dispatch_tag = match result.unwrap_or(MenuBarMouseResult::Ignored) {
            MenuBarMouseResult::Ignored => return Ok(None),
            MenuBarMouseResult::HoverChanged => ProcessOutcome::WidgetConsumed,
            MenuBarMouseResult::Command(command_id) => {
                let enabled = self.with_tui(|tui| tui.commands.is_enabled(command_id));
                if !enabled {
                    return Ok(Some(ProcessOutcome::WidgetConsumed));
                }
                return self
                    .dispatch_tui_command(
                        CommandEvent::application(command_id, Some(view_id)),
                        line,
                    )
                    .map(Some);
            }
        };

        Ok(Some(dispatch_tag))
    }

    /// Drop menu-bar hover and open pull-downs when the pointer leaves every bar region.
    pub(in crate::vm::execute::io::tui) fn sync_menu_bar_hover_outside_pointer(
        &mut self,
        mouse: fpas_std::UiMouse,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if mouse.action != fpas_std::mouse_action_index("Move") {
            return Ok(());
        }

        let redraws = self.with_tui(|tui| {
            let mut redraws = Vec::new();
            for (view_id, widget) in &mut tui.view_widgets {
                let ViewWidget::MenuBar(menu) = widget else {
                    continue;
                };
                if modal_scope.is_some_and(|scope| !scope.contains(view_id)) {
                    continue;
                }
                let Some(rect) = tui.views.rect(*view_id) else {
                    continue;
                };
                if menu.clear_pointer_hover_outside(rect, mouse) {
                    redraws.push(rect);
                }
            }
            for rect in &redraws {
                let _ = tui.session.request_redraw_rect(*rect, line);
            }
            redraws
        });

        if redraws.is_empty() {
            return Ok(());
        }
        Ok(())
    }

    /// Clear menu-bar pointer state after terminal focus loss.
    pub(in crate::vm::execute::io::tui) fn clear_menu_bar_pointer_state(
        &mut self,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let redraws = self.with_tui(|tui| {
            let mut redraws = Vec::new();
            for (view_id, widget) in &mut tui.view_widgets {
                let ViewWidget::MenuBar(menu) = widget else {
                    continue;
                };
                if modal_scope.is_some_and(|scope| !scope.contains(view_id)) {
                    continue;
                }
                let Some(rect) = tui.views.rect(*view_id) else {
                    continue;
                };
                if menu.clear_transient_pointer_state() {
                    redraws.push(rect);
                }
            }
            for rect in &redraws {
                let _ = tui.session.request_redraw_rect(*rect, line);
            }
            redraws
        });

        if redraws.is_empty() {
            return Ok(());
        }
        Ok(())
    }

    /// Open a hovered pull-down when the terminal gains focus after a hover-only pointer path.
    pub(in crate::vm::execute::io::tui) fn try_activate_menu_bar_on_focus_gained(
        &mut self,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::topmost_menu_bar(&tui.views, &tui.view_widgets, modal_scope)
        };
        let Some((view_id, rect)) = hit else {
            return Ok(None);
        };
        let result = self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&view_id)?;
            let ViewWidget::MenuBar(menu) = &mut widget else {
                tui.view_widgets.insert(view_id, widget);
                return Some(MenuBarMouseResult::Ignored);
            };
            let before = menu.damage_rects(rect);
            let result = menu.open_hovered_submenu();
            let after = menu.damage_rects(rect);
            tui.view_widgets.insert(view_id, widget);
            if result != MenuBarMouseResult::Ignored {
                let mut regions = before;
                regions.extend(after);
                Self::request_unique_redraws(tui, &regions, line);
            }
            Some(result)
        });
        if result.unwrap_or(MenuBarMouseResult::Ignored) == MenuBarMouseResult::Ignored {
            return Ok(None);
        }
        Ok(Some(ProcessOutcome::Pointer { handled: true }))
    }

    fn request_unique_redraws(tui: &mut TuiState, regions: &[ViewRect], line: SourceLocation) {
        let mut unique = Vec::new();
        for region in regions {
            if unique.iter().any(|existing: &ViewRect| existing == region) {
                continue;
            }
            unique.push(*region);
        }
        for region in unique {
            let _ = tui.session.request_redraw_rect(region, line);
        }
    }
}
