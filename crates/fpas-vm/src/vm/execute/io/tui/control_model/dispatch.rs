//! Route pointer, keyboard, and paste input to retained controls.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{
    CommandEvent, CommandId, ConsoleKeyEvent, ProcessOutcome, ScrollBarWidget, ScrollViewWidget,
    UiMouse, ViewId, ViewWidget, key_kind_index, mouse_action_index, mouse_button_index,
};

use super::super::widget_target;

enum ControlAction {
    Consumed,
    Command(CommandId),
}

impl Worker {
    pub(in crate::vm::execute::io::tui) fn try_dispatch_control_mouse(
        &mut self,
        mouse: UiMouse,
        scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let down = mouse.action == mouse_action_index("Down")
            && mouse.button == mouse_button_index("Left");
        let scroll_up = mouse.action == mouse_action_index("ScrollUp");
        let scroll_down = mouse.action == mouse_action_index("ScrollDown");
        if !down && !scroll_up && !scroll_down {
            return Ok(None);
        }
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::widget_mouse_hit(&tui.views, &tui.view_widgets, mouse, scope)
        };
        let Some((id, rect, mut widget)) = hit else {
            return Ok(None);
        };
        let state = self.with_tui(|tui| tui.views.state(id));
        if let Some(state) = state {
            widget.sync_view_state(state);
        }
        let action = match &mut widget {
            ViewWidget::Button(v) if v.enabled => {
                v.command_id.map_or(Some(ControlAction::Consumed), |c| {
                    Some(ControlAction::Command(c))
                })
            }
            ViewWidget::InputLine(v) if v.enabled => {
                let index = mouse.x.saturating_sub(1).saturating_sub(rect.x).max(0) as usize
                    + v.scroll_offset();
                v.set_cursor(index);
                Some(ControlAction::Consumed)
            }
            ViewWidget::CheckBox(v) => v.toggle().then(|| {
                v.command_id
                    .map_or(ControlAction::Consumed, ControlAction::Command)
            }),
            ViewWidget::RadioGroup(v) if v.enabled => {
                let row = mouse.y.saturating_sub(1).saturating_sub(rect.y);
                if usize::try_from(row).is_ok_and(|i| v.set_selected(i)) {
                    v.selected_command()
                        .map_or(Some(ControlAction::Consumed), |c| {
                            Some(ControlAction::Command(c))
                        })
                } else {
                    Some(ControlAction::Consumed)
                }
            }
            ViewWidget::ListBox(v) if v.enabled => {
                if scroll_up {
                    v.scroll_by(-1);
                    Some(ControlAction::Consumed)
                } else if scroll_down {
                    v.scroll_by(1);
                    Some(ControlAction::Consumed)
                } else {
                    let row = mouse.y.saturating_sub(1).saturating_sub(rect.y);
                    if usize::try_from(row).is_ok_and(|i| v.select_visible_row(i)) {
                        v.selected_command()
                            .map_or(Some(ControlAction::Consumed), |c| {
                                Some(ControlAction::Command(c))
                            })
                    } else {
                        Some(ControlAction::Consumed)
                    }
                }
            }
            ViewWidget::ScrollBar(v) if v.enabled => {
                scroll_bar_mouse(v, rect, mouse, scroll_up, scroll_down)
            }
            ViewWidget::ScrollView(v) if v.enabled => {
                if scroll_up {
                    v.scroll_by(-1);
                    Some(ControlAction::Consumed)
                } else if scroll_down {
                    v.scroll_by(1);
                    Some(ControlAction::Consumed)
                } else if let Some(hit) = v.scrollbar_hit(rect, mouse.x, mouse.y) {
                    v.apply_scrollbar_hit(rect, hit);
                    Some(ControlAction::Consumed)
                } else {
                    Some(ControlAction::Consumed)
                }
            }
            _ => None,
        };
        self.finish_control_action(id, rect, widget, action, line)
    }

    pub(in crate::vm::execute::io::tui) fn try_dispatch_control_key(
        &mut self,
        key: &ConsoleKeyEvent,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let snapshot = self.with_tui(|tui| {
            let id = tui.views.focused_id()?;
            let view = tui.views.resolved(id)?;
            let widget = tui.view_widgets.get(&id)?.clone();
            Some((id, view.rect, view.state, widget))
        });
        let Some((id, rect, state, mut widget)) = snapshot else {
            return Ok(None);
        };
        widget.sync_view_state(state);
        let enter = key.kind == key_kind_index("Enter");
        let space = key.kind == key_kind_index("Space");
        let action = match &mut widget {
            ViewWidget::Button(v) if v.enabled && (enter || space) => {
                v.command_id.map_or(Some(ControlAction::Consumed), |c| {
                    Some(ControlAction::Command(c))
                })
            }
            ViewWidget::CheckBox(v) if enter || space => v.toggle().then(|| {
                v.command_id
                    .map_or(ControlAction::Consumed, ControlAction::Command)
            }),
            ViewWidget::InputLine(v) if v.enabled => {
                input_key(v, key).then_some(ControlAction::Consumed)
            }
            ViewWidget::RadioGroup(v) if v.enabled => radio_key(v, key),
            ViewWidget::ListBox(v) if v.enabled => list_box_key(v, key),
            ViewWidget::ScrollBar(v) if v.enabled => scroll_control_key(v, key),
            ViewWidget::ScrollView(v) if v.enabled => scroll_control_key(v, key),
            _ => None,
        };
        self.finish_control_action(id, rect, widget, action, line)
    }

    pub(in crate::vm::execute::io::tui) fn try_dispatch_control_paste(
        &mut self,
        text: &str,
        line: SourceLocation,
    ) -> bool {
        self.with_tui(|tui| {
            let Some(id) = tui.views.focused_id() else {
                return false;
            };
            if !tui.views.state(id).is_some_and(|s| s.enabled) {
                return false;
            }
            let Some(ViewWidget::InputLine(input)) = tui.view_widgets.get_mut(&id) else {
                return false;
            };
            input.insert_str(text);
            if let Some(rect) = tui.views.rect(id) {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
            true
        })
    }

    fn finish_control_action(
        &mut self,
        id: ViewId,
        rect: fpas_std::ViewRect,
        widget: ViewWidget,
        action: Option<ControlAction>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let Some(action) = action else {
            return Ok(None);
        };
        self.with_tui(|tui| {
            tui.view_widgets.insert(id, widget);
            let _ = tui.session.request_redraw_rect(rect, line);
        });
        match action {
            ControlAction::Consumed => Ok(Some(ProcessOutcome::WidgetConsumed)),
            ControlAction::Command(command) => {
                if !self.with_tui(|tui| tui.commands.is_enabled(command)) {
                    return Ok(Some(ProcessOutcome::WidgetConsumed));
                }
                self.dispatch_tui_command(CommandEvent::application(command, Some(id)), line)
                    .map(Some)
            }
        }
    }
}

fn input_key(input: &mut fpas_std::InputLineWidget, key: &ConsoleKeyEvent) -> bool {
    if key.ctrl || key.alt || key.meta {
        return false;
    }
    match key.kind {
        k if k == key_kind_index("Character") => {
            input.insert_char(key.ch);
            true
        }
        k if k == key_kind_index("Backspace") => {
            input.backspace();
            true
        }
        k if k == key_kind_index("Delete") => {
            input.delete();
            true
        }
        k if k == key_kind_index("Left") => {
            input.move_cursor_left();
            true
        }
        k if k == key_kind_index("Right") => {
            input.move_cursor_right();
            true
        }
        k if k == key_kind_index("Home") => {
            input.set_cursor(0);
            true
        }
        k if k == key_kind_index("End") => {
            input.set_cursor(usize::MAX);
            true
        }
        _ => false,
    }
}

fn radio_key(
    group: &mut fpas_std::RadioGroupWidget,
    key: &ConsoleKeyEvent,
) -> Option<ControlAction> {
    if key.kind == key_kind_index("Up") || key.kind == key_kind_index("Left") {
        group.focus_prev();
        return Some(ControlAction::Consumed);
    }
    if key.kind == key_kind_index("Down") || key.kind == key_kind_index("Right") {
        group.focus_next();
        return Some(ControlAction::Consumed);
    }
    if key.kind == key_kind_index("Enter") || key.kind == key_kind_index("Space") {
        group.select_focused();
        return group
            .selected_command()
            .map_or(Some(ControlAction::Consumed), |c| {
                Some(ControlAction::Command(c))
            });
    }
    None
}

fn list_box_key(
    list: &mut fpas_std::ListBoxWidget,
    key: &ConsoleKeyEvent,
) -> Option<ControlAction> {
    let changed = if key.kind == key_kind_index("Up") {
        list.move_selection(false)
    } else if key.kind == key_kind_index("Down") {
        list.move_selection(true)
    } else if key.kind == key_kind_index("Home") {
        list.select_edge(false)
    } else if key.kind == key_kind_index("End") {
        list.select_edge(true)
    } else {
        false
    };
    if changed {
        return Some(ControlAction::Consumed);
    }
    if key.kind == key_kind_index("Enter") || key.kind == key_kind_index("Space") {
        return list
            .selected_command()
            .map_or(Some(ControlAction::Consumed), |c| {
                Some(ControlAction::Command(c))
            });
    }
    None
}

fn scroll_bar_mouse(
    bar: &mut fpas_std::ScrollBarWidget,
    rect: fpas_std::ViewRect,
    mouse: UiMouse,
    scroll_up: bool,
    scroll_down: bool,
) -> Option<ControlAction> {
    if scroll_up {
        bar.scroll_by(-1);
        return Some(ControlAction::Consumed);
    }
    if scroll_down {
        bar.scroll_by(1);
        return Some(ControlAction::Consumed);
    }
    if let Some(hit) = bar.hit_test(rect, mouse.x, mouse.y) {
        bar.apply_hit(hit);
    }
    Some(ControlAction::Consumed)
}

fn scroll_control_key(
    scroll: &mut impl ScrollControl,
    key: &ConsoleKeyEvent,
) -> Option<ControlAction> {
    let changed = if key.kind == key_kind_index("Up") {
        scroll.scroll_by(-1)
    } else if key.kind == key_kind_index("Down") {
        scroll.scroll_by(1)
    } else if key.kind == key_kind_index("PageUp") {
        scroll.scroll_page(false)
    } else if key.kind == key_kind_index("PageDown") {
        scroll.scroll_page(true)
    } else if key.kind == key_kind_index("Home") {
        scroll.set_offset(0)
    } else if key.kind == key_kind_index("End") {
        scroll.set_offset(usize::MAX)
    } else {
        false
    };
    changed.then_some(ControlAction::Consumed)
}

trait ScrollControl {
    fn scroll_by(&mut self, delta: i64) -> bool;
    fn scroll_page(&mut self, forward: bool) -> bool;
    fn set_offset(&mut self, offset: usize) -> bool;
}

impl ScrollControl for fpas_std::ScrollBarWidget {
    fn scroll_by(&mut self, delta: i64) -> bool {
        ScrollBarWidget::scroll_by(self, delta)
    }
    fn scroll_page(&mut self, forward: bool) -> bool {
        ScrollBarWidget::scroll_page(self, forward)
    }
    fn set_offset(&mut self, offset: usize) -> bool {
        ScrollBarWidget::set_offset(self, offset)
    }
}

impl ScrollControl for fpas_std::ScrollViewWidget {
    fn scroll_by(&mut self, delta: i64) -> bool {
        ScrollViewWidget::scroll_by(self, delta)
    }
    fn scroll_page(&mut self, forward: bool) -> bool {
        ScrollViewWidget::scroll_page(self, forward)
    }
    fn set_offset(&mut self, offset: usize) -> bool {
        ScrollViewWidget::set_offset(self, offset)
    }
}
