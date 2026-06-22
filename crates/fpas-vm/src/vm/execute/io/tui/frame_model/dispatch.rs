//! Route pointer events to frame chrome move, resize, scroll, and activation.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{
    ConsoleKeyEvent, FrameChromeHit, FrameScrollHit, ProcessOutcome, ScrollBarHit, UiMouse, ViewId,
    ViewRect, mouse_action_index, mouse_button_index,
};

impl Worker {
    /// Handle an active frame drag or a new frame chrome hit before widget routing.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_frame_mouse(
        &mut self,
        mouse: UiMouse,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let up =
            mouse.action == mouse_action_index("Up") && mouse.button == mouse_button_index("Left");
        let move_ = mouse.action == mouse_action_index("Move");
        let down = mouse.action == mouse_action_index("Down")
            && mouse.button == mouse_button_index("Left");

        if self.with_tui(|tui| tui.views.frame_scroll_interaction().is_some()) {
            return self.dispatch_active_frame_scroll_interaction(mouse, up, move_, line);
        }

        if self.with_tui(|tui| tui.views.window_interaction().is_some()) {
            return self.dispatch_active_frame_interaction(mouse, up, move_, line);
        }

        if !down {
            return Ok(None);
        }

        let x = mouse.x.saturating_sub(1);
        let y = mouse.y.saturating_sub(1);
        let root = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.views
                .topmost_view_at(x, y, modal_scope)
                .and_then(|hit| tui.views.frame_root_of(hit))
        };
        let Some(root) = root else {
            return Ok(None);
        };

        if let Some(scroll_hit) = self.with_tui(|tui| tui.views.frame_scroll_hit_at(root, x, y)) {
            let _ = self.with_tui(|tui| tui.views.activate_root(root));
            if scroll_hit == FrameScrollHit::Vertical(ScrollBarHit::Thumb)
                || scroll_hit == FrameScrollHit::Horizontal(ScrollBarHit::Thumb)
            {
                let began = self.with_tui(|tui| {
                    tui.views
                        .begin_frame_scroll_thumb_drag(root, scroll_hit, x, y)
                });
                if began {
                    self.request_frame_root_damage(None, root, line)?;
                    return Ok(Some(ProcessOutcome::WidgetConsumed));
                }
            } else {
                let changed =
                    self.with_tui(|tui| tui.views.apply_frame_scroll_hit(root, scroll_hit));
                if changed {
                    self.sync_frame_widget_scroll(root);
                    self.request_frame_root_damage(None, root, line)?;
                }
                return Ok(Some(ProcessOutcome::WidgetConsumed));
            }
        }

        let hit = self.with_tui(|tui| tui.views.frame_chrome_hit_at(root, x, y));
        let began = self.with_tui(|tui| match hit {
            FrameChromeHit::Close | FrameChromeHit::Zoom | FrameChromeHit::ZoomBack => false,
            FrameChromeHit::Move => tui.views.begin_frame_move(root, x, y),
            FrameChromeHit::Resize(_) => tui.views.begin_frame_resize(root, x, y),
            FrameChromeHit::None => false,
        });
        if began {
            let _ = self.with_tui(|tui| tui.views.activate_root(root));
            self.request_frame_root_damage(None, root, line)?;
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }

        if matches!(
            hit,
            FrameChromeHit::Close | FrameChromeHit::Zoom | FrameChromeHit::ZoomBack
        ) {
            let _ = self.with_tui(|tui| tui.views.activate_root(root));
            let command = match hit {
                FrameChromeHit::Close => fpas_std::CommandEvent::resolve(
                    fpas_std::CommandId(fpas_std::COMMAND_ID_CLOSE),
                    Some(root),
                ),
                FrameChromeHit::Zoom => fpas_std::CommandEvent::resolve(
                    fpas_std::CommandId(fpas_std::COMMAND_ID_ZOOM),
                    Some(root),
                ),
                FrameChromeHit::ZoomBack => fpas_std::CommandEvent::resolve(
                    fpas_std::CommandId(fpas_std::COMMAND_ID_ZOOM_BACK),
                    Some(root),
                ),
                _ => unreachable!("handled above"),
            };
            return self.dispatch_tui_command(command, line).map(Some);
        }

        if hit == FrameChromeHit::None
            && self.with_tui(|tui| {
                tui.views
                    .frame_root_state(root)
                    .is_some_and(|state| state.geometry.title_bar.contains_point(x, y))
            })
        {
            let _ = self.with_tui(|tui| tui.views.activate_root(root));
            self.request_frame_root_damage(None, root, line)?;
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }

        Ok(None)
    }

    /// Scroll a frame root under the pointer when wheel events were not consumed.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_frame_wheel(
        &mut self,
        mouse: UiMouse,
        modal_scope: Option<&[ViewId]>,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let scroll_up = mouse.action == mouse_action_index("ScrollUp");
        let scroll_down = mouse.action == mouse_action_index("ScrollDown");
        if !scroll_up && !scroll_down {
            return Ok(None);
        }
        let x = mouse.x.saturating_sub(1);
        let y = mouse.y.saturating_sub(1);
        let root = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.views
                .topmost_view_at(x, y, modal_scope)
                .and_then(|hit| tui.views.frame_root_of(hit))
        };
        let Some(root) = root else {
            return Ok(None);
        };
        let in_viewport = self.with_tui(|tui| {
            tui.views
                .frame_root_state(root)
                .is_some_and(|state| state.geometry.view.contains_point(x, y))
        });
        if !in_viewport {
            return Ok(None);
        }
        let delta_y = if scroll_up { -1 } else { 1 };
        let changed = self.with_tui(|tui| tui.views.scroll_frame(root, 0, delta_y));
        if changed {
            self.sync_frame_widget_scroll(root);
            self.request_frame_root_damage(None, root, line)?;
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        Ok(None)
    }

    /// Scroll the frame root containing focus when arrow/page keys were not consumed.
    pub(in crate::vm::execute::io::tui) fn try_dispatch_frame_key(
        &mut self,
        key: &ConsoleKeyEvent,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let focused = self.with_tui(|tui| tui.views.focused_id());
        let Some(focused) = focused else {
            return Ok(None);
        };
        let changed = self.with_tui(|tui| tui.views.scroll_frame_key(focused, key.clone()));
        if !changed {
            return Ok(None);
        }
        let root = self.with_tui(|tui| tui.views.frame_root_of(focused));
        let Some(root) = root else {
            return Ok(None);
        };
        self.sync_frame_widget_scroll(root);
        self.request_frame_root_damage(None, root, line)?;
        Ok(Some(ProcessOutcome::WidgetConsumed))
    }

    fn dispatch_active_frame_scroll_interaction(
        &mut self,
        mouse: UiMouse,
        up: bool,
        move_: bool,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let root = self.with_tui(|tui| tui.views.frame_scroll_interaction().map(|i| i.root));
        let Some(root) = root else {
            return Ok(None);
        };
        let x = mouse.x.saturating_sub(1);
        let y = mouse.y.saturating_sub(1);
        if move_ {
            let changed = self.with_tui(|tui| tui.views.drag_frame_scroll_thumb(x, y));
            if changed {
                self.sync_frame_widget_scroll(root);
                self.request_frame_root_damage(None, root, line)?;
            }
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        if up {
            let _ = self.with_tui(|tui| tui.views.end_frame_scroll_interaction());
            self.request_frame_root_damage(None, root, line)?;
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        Ok(None)
    }

    fn dispatch_active_frame_interaction(
        &mut self,
        mouse: UiMouse,
        up: bool,
        move_: bool,
        line: SourceLocation,
    ) -> Result<Option<ProcessOutcome>, VmError> {
        let root = self.with_tui(|tui| tui.views.window_interaction().map(|i| i.root));
        let Some(root) = root else {
            return Ok(None);
        };
        let x = mouse.x.saturating_sub(1);
        let y = mouse.y.saturating_sub(1);
        if move_ {
            let previous = self.with_tui(|tui| Worker::frame_damage_rects(tui, root));
            let changed = self.with_tui(|tui| tui.views.drag_frame_interaction(x, y));
            if changed {
                self.request_frame_root_damage(Some(previous), root, line)?;
            }
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        if up {
            let previous = self.with_tui(|tui| Worker::frame_damage_rects(tui, root));
            let _ = self.with_tui(|tui| tui.views.end_frame_interaction());
            self.request_frame_root_damage(Some(previous), root, line)?;
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        Ok(None)
    }

    pub(in crate::vm::execute::io::tui) fn sync_frame_widget_scroll(&mut self, root: ViewId) {
        let snapshot = self.with_tui(|tui| {
            let state = tui.views.frame_root_state(root)?;
            let widget = tui.view_widgets.get(&root).cloned()?;
            Some((state.scroll_x, state.scroll_y, state.content_size, widget))
        });
        let Some((scroll_x, scroll_y, content_size, mut widget)) = snapshot else {
            return;
        };
        if let fpas_std::ViewWidget::Frame(frame) = &mut widget {
            frame.scroll_x = scroll_x;
            frame.scroll_y = scroll_y;
            frame.content_size = content_size;
        }
        let _ = self.with_tui(|tui| {
            tui.view_widgets.insert(root, widget);
        });
    }

    fn request_frame_root_damage(
        &mut self,
        previous: Option<Vec<ViewRect>>,
        root: ViewId,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.with_tui(|tui| {
            Worker::request_frame_subtree_damage(tui, previous.as_deref(), root, line);
        });
        Ok(())
    }
}
