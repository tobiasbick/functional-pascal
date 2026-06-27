//! Route pointer, keyboard, and paste input to retained controls.
//!
//! **Documentation:** `docs/pascal/std/tui/app/controls.md`

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{
    CommandEvent, CommandId, ConsoleKeyEvent, MemoWidget, ProcessOutcome, ScrollBarWidget,
    ScrollViewWidget, UiMouse, ViewId, ViewWidget, key_kind_index, mouse_action_index,
    mouse_button_index,
};

use super::super::widget_target;

enum ControlAction {
    Consumed,
    CaptureThumb,
    CapturePress,
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
        let up =
            mouse.action == mouse_action_index("Up") && mouse.button == mouse_button_index("Left");
        let move_ = mouse.action == mouse_action_index("Move");
        let scroll_up = mouse.action == mouse_action_index("ScrollUp");
        let scroll_down = mouse.action == mouse_action_index("ScrollDown");
        if (up || move_)
            && let Some(outcome) =
                self.try_dispatch_scroll_thumb_drag(mouse, scope, up, move_, line)?
        {
            return Ok(Some(outcome));
        }
        if up && let Some(outcome) = self.try_dispatch_pressed_button_release(mouse, line)? {
            return Ok(Some(outcome));
        }
        if !down && !scroll_up && !scroll_down {
            return Ok(None);
        }
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::widget_mouse_hit(&tui.views, &tui.view_widgets, mouse, scope)
        };
        let Some((id, rect)) = hit else {
            return Ok(None);
        };
        let action = self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&id)?;
            if let Some(state) = tui.views.state(id) {
                widget.sync_view_state(state);
            }
            let action = match &mut widget {
                ViewWidget::Button(v) if v.enabled => down.then_some(ControlAction::CapturePress),
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
                    scroll_bar_mouse(v, rect, mouse, scroll_up, scroll_down, down)
                }
                ViewWidget::ScrollView(v) if v.enabled => {
                    scroll_view_mouse(v, rect, mouse, scroll_up, scroll_down, down)
                }
                ViewWidget::Memo(v) if v.enabled => {
                    memo_mouse(v, rect, mouse, scroll_up, scroll_down, down)
                }
                _ => None,
            };
            let capture_thumb = matches!(
                (&widget, &action),
                (ViewWidget::ScrollBar(v), Some(ControlAction::CaptureThumb))
                    if v.thumb_drag_active(),
            ) || matches!(
                (&widget, &action),
                (ViewWidget::ScrollView(v), Some(ControlAction::CaptureThumb))
                    if v.thumb_drag_active(),
            ) || matches!(
                (&widget, &action),
                (ViewWidget::Memo(v), Some(ControlAction::CaptureThumb)) if v.thumb_drag_active(),
            );
            tui.view_widgets.insert(id, widget);
            Some((action, capture_thumb))
        });
        let Some((action, capture_thumb)) = action else {
            return Ok(None);
        };
        self.finish_control_action(id, rect, action, capture_thumb, line)
    }

    fn try_dispatch_pressed_button_release(
        &mut self,
        mouse: UiMouse,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let snapshot = self.with_tui(|tui| {
            let id = tui.views.pressed_pointer()?;
            let view = tui.views.resolved(id)?;
            let mut widget = tui.view_widgets.remove(&id)?;
            let command = match &mut widget {
                ViewWidget::Button(button) if button.enabled => view
                    .rect
                    .contains_console_mouse(mouse.x, mouse.y)
                    .then_some(button.command_id),
                _ => None,
            };
            tui.view_widgets.insert(id, widget);
            tui.views.end_pointer_press();
            Some((id, view.rect, command.flatten()))
        });
        let Some((id, rect, command)) = snapshot else {
            return Ok(None);
        };
        let action = command.map_or(Some(ControlAction::Consumed), |c| {
            Some(ControlAction::Command(c))
        });
        self.finish_control_action(id, rect, action, false, line)
    }

    fn try_dispatch_scroll_thumb_drag(
        &mut self,
        mouse: UiMouse,
        scope: Option<&[ViewId]>,
        up: bool,
        move_: bool,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let hit = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            widget_target::widget_mouse_hit(&tui.views, &tui.view_widgets, mouse, scope)
        };
        let Some((id, rect)) = hit else {
            return Ok(None);
        };
        let handled = self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&id)?;
            if let Some(state) = tui.views.state(id) {
                widget.sync_view_state(state);
            }
            let active = match &widget {
                ViewWidget::ScrollBar(v) => v.thumb_drag_active(),
                ViewWidget::ScrollView(v) => v.thumb_drag_active(),
                ViewWidget::Memo(v) => v.thumb_drag_active(),
                _ => false,
            };
            if !active {
                tui.view_widgets.insert(id, widget);
                return Some(false);
            }
            let changed = match &mut widget {
                ViewWidget::ScrollBar(v) if move_ => v.drag_thumb(rect, mouse.x, mouse.y),
                ViewWidget::ScrollView(v) if move_ => v.drag_thumb(rect, mouse.x, mouse.y),
                ViewWidget::Memo(v) if move_ => v.drag_thumb(rect, mouse.x, mouse.y),
                ViewWidget::ScrollBar(v) if up => {
                    v.end_thumb_drag();
                    true
                }
                ViewWidget::ScrollView(v) if up => {
                    v.end_thumb_drag();
                    true
                }
                ViewWidget::Memo(v) if up => {
                    v.end_thumb_drag();
                    true
                }
                _ => false,
            };
            tui.view_widgets.insert(id, widget);
            if up {
                tui.views.release_pointer();
            }
            if changed {
                let _ = tui.session.request_redraw_rect(rect, line);
            }
            Some(true)
        });
        if !handled.unwrap_or(false) {
            return Ok(None);
        }
        Ok(Some(ProcessOutcome::WidgetConsumed))
    }

    pub(in crate::vm::execute::io::tui) fn try_dispatch_control_key(
        &mut self,
        key: &ConsoleKeyEvent,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let snapshot = self.with_tui(|tui| {
            let id = tui.views.focused_id()?;
            let view = tui.views.resolved(id)?;
            Some((id, view.rect, view.state))
        });
        let Some((id, rect, state)) = snapshot else {
            return Ok(None);
        };
        let action = self.with_tui(|tui| {
            let mut widget = tui.view_widgets.remove(&id)?;
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
                ViewWidget::Memo(v) if v.enabled => {
                    memo_key(v, key).then_some(ControlAction::Consumed)
                }
                ViewWidget::RadioGroup(v) if v.enabled => radio_key(v, key),
                ViewWidget::ListBox(v) if v.enabled => list_box_key(v, key),
                ViewWidget::ScrollBar(v) if v.enabled => scroll_control_key(v, key),
                ViewWidget::ScrollView(v) if v.enabled => scroll_control_key(v, key),
                _ => None,
            };
            tui.view_widgets.insert(id, widget);
            action
        });
        self.finish_control_action(id, rect, action, false, line)
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
                let Some(ViewWidget::Memo(memo)) = tui.view_widgets.get_mut(&id) else {
                    return false;
                };
                memo.insert_str(text);
                if let Some(rect) = tui.views.rect(id) {
                    let _ = tui.session.request_redraw_rect(rect, line);
                }
                return true;
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
        action: Option<ControlAction>,
        capture_thumb: bool,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let Some(action) = action else {
            return Ok(None);
        };
        self.with_tui(|tui| {
            if capture_thumb {
                tui.views.capture_pointer(id);
            }
            if matches!(action, ControlAction::CapturePress) {
                tui.views.begin_pointer_press(id);
            }
            let _ = tui.session.request_redraw_rect(rect, line);
        });
        match action {
            ControlAction::Consumed | ControlAction::CaptureThumb | ControlAction::CapturePress => {
                Ok(Some(ProcessOutcome::WidgetConsumed))
            }
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
    down: bool,
) -> Option<ControlAction> {
    if scroll_up {
        bar.scroll_by(-1);
        return Some(ControlAction::Consumed);
    }
    if scroll_down {
        bar.scroll_by(1);
        return Some(ControlAction::Consumed);
    }
    if down {
        if bar.begin_thumb_drag(rect, mouse.x, mouse.y) {
            return Some(ControlAction::CaptureThumb);
        }
        if let Some(hit) = bar.hit_test(rect, mouse.x, mouse.y) {
            bar.apply_hit(hit);
        }
    }
    Some(ControlAction::Consumed)
}

fn memo_mouse(
    memo: &mut MemoWidget,
    rect: fpas_std::ViewRect,
    mouse: UiMouse,
    scroll_up: bool,
    scroll_down: bool,
    down: bool,
) -> Option<ControlAction> {
    if scroll_up {
        memo.scroll_by(-1);
        return Some(ControlAction::Consumed);
    }
    if scroll_down {
        memo.scroll_by(1);
        return Some(ControlAction::Consumed);
    }
    if down {
        if memo.begin_thumb_drag(rect, mouse.x, mouse.y) {
            return Some(ControlAction::CaptureThumb);
        }
        if let Some(hit) = memo.scrollbar_hit(rect, mouse.x, mouse.y) {
            memo.apply_scrollbar_hit(rect, hit);
            return Some(ControlAction::Consumed);
        }
        let content = memo.content_rect(rect);
        memo.set_cursor_from_click(
            content,
            mouse.x.saturating_sub(1),
            mouse.y.saturating_sub(1),
        );
        return Some(ControlAction::Consumed);
    }
    None
}

fn memo_key(memo: &mut MemoWidget, key: &ConsoleKeyEvent) -> bool {
    if key.ctrl || key.alt || key.meta {
        return false;
    }
    let extend = key.shift;
    match key.kind {
        k if k == key_kind_index("Character") => {
            memo.insert_char(key.ch);
            true
        }
        k if k == key_kind_index("Backspace") => {
            memo.backspace();
            true
        }
        k if k == key_kind_index("Delete") => {
            memo.delete();
            true
        }
        k if k == key_kind_index("Enter") => {
            memo.insert_char('\n');
            true
        }
        k if k == key_kind_index("Left") => {
            memo.move_cursor(0, -1, extend);
            true
        }
        k if k == key_kind_index("Right") => {
            memo.move_cursor(0, 1, extend);
            true
        }
        k if k == key_kind_index("Up") => {
            memo.move_cursor(-1, 0, extend);
            true
        }
        k if k == key_kind_index("Down") => {
            memo.move_cursor(1, 0, extend);
            true
        }
        k if k == key_kind_index("Home") => {
            memo.move_cursor_line_edge(false, extend);
            true
        }
        k if k == key_kind_index("End") => {
            memo.move_cursor_line_edge(true, extend);
            true
        }
        k if k == key_kind_index("PageUp") => memo.scroll_page(false),
        k if k == key_kind_index("PageDown") => memo.scroll_page(true),
        _ => false,
    }
}

fn scroll_view_mouse(
    view: &mut fpas_std::ScrollViewWidget,
    rect: fpas_std::ViewRect,
    mouse: UiMouse,
    scroll_up: bool,
    scroll_down: bool,
    down: bool,
) -> Option<ControlAction> {
    if scroll_up {
        view.scroll_by(-1);
        return Some(ControlAction::Consumed);
    }
    if scroll_down {
        view.scroll_by(1);
        return Some(ControlAction::Consumed);
    }
    if down {
        if view.begin_thumb_drag(rect, mouse.x, mouse.y) {
            return Some(ControlAction::CaptureThumb);
        }
        if let Some(hit) = view.scrollbar_hit(rect, mouse.x, mouse.y) {
            view.apply_scrollbar_hit(rect, hit);
        }
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
