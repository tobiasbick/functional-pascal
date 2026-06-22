//! Route pointer events to frame chrome move, resize, and activation.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;
use fpas_std::{
    FrameChromeHit, ProcessOutcome, UiMouse, ViewId, ViewRect, mouse_action_index,
    mouse_button_index,
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
            let before = self.with_tui(|tui| tui.views.rect(root));
            let changed = self.with_tui(|tui| tui.views.drag_frame_interaction(x, y));
            if changed {
                self.request_frame_root_damage(before, root, line)?;
            }
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        if up {
            let before = self.with_tui(|tui| tui.views.rect(root));
            let _ = self.with_tui(|tui| tui.views.end_frame_interaction());
            self.request_frame_root_damage(before, root, line)?;
            return Ok(Some(ProcessOutcome::WidgetConsumed));
        }
        Ok(None)
    }

    fn request_frame_root_damage(
        &mut self,
        before: Option<ViewRect>,
        root: ViewId,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.with_tui(|tui| {
            let after = tui.views.rect(root);
            match (before, after) {
                (Some(b), Some(a)) => {
                    let _ = tui.session.request_redraw_rect(b.union(a), line);
                }
                (_, Some(a)) => {
                    let _ = tui.session.request_redraw_rect(a, line);
                }
                _ => {
                    let _ = tui.session.request_redraw(line);
                }
            }
        });
        Ok(())
    }
}
