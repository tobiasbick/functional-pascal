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

        let Some((view_id, rect, mut widget)) = hit else {
            return Ok(None);
        };

        let before = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };

        let result = match &mut widget {
            ViewWidget::MenuBar(menu) => menu.handle_mouse(rect, mouse),
            ViewWidget::SolidFill(_)
            | ViewWidget::StatusBar(_)
            | ViewWidget::Label(_)
            | ViewWidget::Button(_)
            | ViewWidget::InputLine(_)
            | ViewWidget::CheckBox(_)
            | ViewWidget::RadioGroup(_) => MenuBarMouseResult::Ignored,
            ViewWidget::ListBox(_) => MenuBarMouseResult::Ignored,
        };

        let after = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };

        let dispatch_tag = match result {
            MenuBarMouseResult::Ignored => return Ok(None),
            MenuBarMouseResult::HoverChanged => {
                self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                });
                ProcessOutcome::Pointer { handled: true }
            }
            MenuBarMouseResult::Command(command_id) => {
                let enabled = self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                    tui.commands.is_enabled(command_id)
                });
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

        let Some((view_id, rect, mut widget)) = hit else {
            return Ok(None);
        };

        let ViewWidget::MenuBar(menu) = &widget else {
            return Ok(None);
        };
        let before = menu.damage_rects(rect);

        let result = match &mut widget {
            ViewWidget::MenuBar(menu) => menu.handle_key(&key),
            _ => unreachable!(),
        };
        let after = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };
        let dispatch_tag = match result {
            MenuBarMouseResult::Ignored => return Ok(None),
            MenuBarMouseResult::HoverChanged => {
                self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                });
                ProcessOutcome::WidgetConsumed
            }
            MenuBarMouseResult::Command(command_id) => {
                let enabled = self.with_tui(|tui| {
                    tui.view_widgets.insert(view_id, widget);
                    let mut regions = before;
                    regions.extend(after);
                    Self::request_unique_redraws(tui, &regions, line);
                    tui.commands.is_enabled(command_id)
                });
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

        let updates = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.view_widgets
                .iter()
                .filter_map(|(view_id, widget)| {
                    let ViewWidget::MenuBar(menu) = widget else {
                        return None;
                    };
                    if modal_scope.is_some_and(|scope| !scope.contains(view_id)) {
                        return None;
                    }
                    let rect = tui.views.rect(*view_id)?;
                    let mut menu = menu.clone();
                    menu.clear_pointer_hover_outside(rect, mouse)
                        .then_some((*view_id, rect, menu))
                })
                .collect::<Vec<_>>()
        };

        if updates.is_empty() {
            return Ok(());
        }

        self.with_tui(|tui| {
            for (view_id, rect, widget) in updates {
                tui.view_widgets
                    .insert(view_id, ViewWidget::MenuBar(widget));
                let _ = tui.session.request_redraw_rect(rect, line);
            }
        });
        Ok(())
    }

    /// Clear menu-bar pointer state after terminal focus loss.
    pub(in crate::vm::execute::io::tui) fn clear_menu_bar_pointer_state(
        &mut self,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let updates = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.view_widgets
                .iter()
                .filter_map(|(view_id, widget)| {
                    let ViewWidget::MenuBar(menu) = widget else {
                        return None;
                    };
                    if modal_scope.is_some_and(|scope| !scope.contains(view_id)) {
                        return None;
                    }
                    let rect = tui.views.rect(*view_id)?;
                    let mut menu = menu.clone();
                    menu.clear_transient_pointer_state()
                        .then_some((*view_id, rect, menu))
                })
                .collect::<Vec<_>>()
        };

        if updates.is_empty() {
            return Ok(());
        }

        self.with_tui(|tui| {
            for (view_id, rect, widget) in updates {
                tui.view_widgets
                    .insert(view_id, ViewWidget::MenuBar(widget));
                let _ = tui.session.request_redraw_rect(rect, line);
            }
        });
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
        let Some((view_id, rect, mut widget)) = hit else {
            return Ok(None);
        };
        let ViewWidget::MenuBar(menu) = &widget else {
            return Ok(None);
        };
        let before = menu.damage_rects(rect);
        let result = match &mut widget {
            ViewWidget::MenuBar(menu) => menu.open_hovered_submenu(),
            _ => unreachable!(),
        };
        let after = match &widget {
            ViewWidget::MenuBar(menu) => menu.damage_rects(rect),
            _ => vec![rect],
        };
        if result == MenuBarMouseResult::Ignored {
            return Ok(None);
        }
        self.with_tui(|tui| {
            tui.view_widgets.insert(view_id, widget);
            let mut regions = before;
            regions.extend(after);
            Self::request_unique_redraws(tui, &regions, line);
        });
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
