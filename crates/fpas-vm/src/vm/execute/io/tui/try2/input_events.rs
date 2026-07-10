//! Try-2 Turbo Vision keyboard and mouse fallback dispatch.
//!
//! **Documentation:** `docs/pascal/std/tui/app/handlers.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use fpas_std::{
    ConsoleEvent, ConsoleKeyEvent, key_event::key_kind_index, mouse_action_index,
    mouse_button_index,
};
use turbo_vision::core::event::{
    Event, EventType, KB_BACKSPACE, KB_DEL, KB_DOWN, KB_END, KB_ENTER, KB_ESC, KB_ESC_ESC, KB_F1,
    KB_F2, KB_F3, KB_F4, KB_F5, KB_F6, KB_F7, KB_F8, KB_F9, KB_F10, KB_F11, KB_F12, KB_HOME,
    KB_INS, KB_LEFT, KB_PGDN, KB_PGUP, KB_RIGHT, KB_SHIFT_TAB, KB_TAB, KB_UP, MB_LEFT_BUTTON,
    MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON, MouseEvent,
};

/// Map one Turbo Vision keyboard event to `Std.Console.KeyEvent`.
pub(in crate::vm::execute::io::tui) fn turbo_vision_keyboard_to_console_key(
    event: &Event,
) -> ConsoleKeyEvent {
    let bits = event.key_modifiers.bits();
    let shift = bits & 0x01 != 0;
    let ctrl = bits & 0x02 != 0;
    let alt = bits & 0x04 != 0;
    let meta = bits & 0x20 != 0;
    let (kind, ch) = turbo_vision_key_code_to_kind_and_char(event.key_code);
    ConsoleKeyEvent::new(kind, ch, shift, ctrl, alt, meta)
}

fn turbo_vision_key_code_to_kind_and_char(code: u16) -> (usize, char) {
    let kind = |name: &str| key_kind_index(name);
    match code {
        KB_ESC | KB_ESC_ESC => (kind("Escape"), '\0'),
        KB_TAB | KB_SHIFT_TAB => (kind("Tab"), '\0'),
        KB_ENTER => (kind("Enter"), '\0'),
        KB_BACKSPACE => (kind("Backspace"), '\0'),
        KB_UP => (kind("Up"), '\0'),
        KB_DOWN => (kind("Down"), '\0'),
        KB_LEFT => (kind("Left"), '\0'),
        KB_RIGHT => (kind("Right"), '\0'),
        KB_HOME => (kind("Home"), '\0'),
        KB_END => (kind("End"), '\0'),
        KB_PGUP => (kind("PageUp"), '\0'),
        KB_PGDN => (kind("PageDown"), '\0'),
        KB_INS => (kind("Insert"), '\0'),
        KB_DEL => (kind("Delete"), '\0'),
        KB_F1 => (kind("F1"), '\0'),
        KB_F2 => (kind("F2"), '\0'),
        KB_F3 => (kind("F3"), '\0'),
        KB_F4 => (kind("F4"), '\0'),
        KB_F5 => (kind("F5"), '\0'),
        KB_F6 => (kind("F6"), '\0'),
        KB_F7 => (kind("F7"), '\0'),
        KB_F8 => (kind("F8"), '\0'),
        KB_F9 => (kind("F9"), '\0'),
        KB_F10 => (kind("F10"), '\0'),
        KB_F11 => (kind("F11"), '\0'),
        KB_F12 => (kind("F12"), '\0'),
        c if (0x20..=0x7E).contains(&c) => (kind("Character"), c as u8 as char),
        c if (0x01..=0x1A).contains(&c) => {
            let ch = char::from(b'a' + (c as u8) - 1);
            (kind("Character"), ch)
        }
        _ => (kind("Unknown"), '\0'),
    }
}

/// Map one unhandled Turbo Vision mouse event to `Std.Console.Event`.
pub(in crate::vm::execute::io::tui) fn turbo_vision_mouse_to_console_event(
    event: &Event,
) -> Option<ConsoleEvent> {
    let action = match event.what {
        EventType::MouseDown => mouse_action_index("Down"),
        EventType::MouseUp => mouse_action_index("Up"),
        EventType::MouseMove | EventType::MouseAuto => mouse_action_index("Move"),
        EventType::MouseWheelUp => mouse_action_index("ScrollUp"),
        EventType::MouseWheelDown => mouse_action_index("ScrollDown"),
        _ => return None,
    };
    let button = turbo_vision_mouse_button(event.mouse);
    let x = i64::from(event.mouse.pos.x);
    let y = i64::from(event.mouse.pos.y);
    Some(ConsoleEvent::mouse(
        action, button, x, y, false, false, false, false,
    ))
}

fn turbo_vision_mouse_button(mouse: MouseEvent) -> usize {
    if mouse.buttons & MB_LEFT_BUTTON != 0 {
        mouse_button_index("Left")
    } else if mouse.buttons & MB_RIGHT_BUTTON != 0 {
        mouse_button_index("Right")
    } else if mouse.buttons & MB_MIDDLE_BUTTON != 0 {
        mouse_button_index("Middle")
    } else {
        mouse_button_index("None")
    }
}

impl Worker {
    /// Dispatch keyboard or mouse events left unhandled by Turbo Vision into opt-in FPAS hooks.
    pub(in crate::vm::execute::io::tui) fn dispatch_turbo_vision_unhandled_input(
        &mut self,
        event: &mut Event,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        match event.what {
            EventType::Keyboard => self.dispatch_turbo_vision_unhandled_keyboard(event, line),
            EventType::MouseDown
            | EventType::MouseUp
            | EventType::MouseMove
            | EventType::MouseAuto
            | EventType::MouseWheelUp
            | EventType::MouseWheelDown => self.dispatch_turbo_vision_unhandled_mouse(event, line),
            _ => Ok(()),
        }
    }

    fn dispatch_turbo_vision_unhandled_keyboard(
        &mut self,
        event: &mut Event,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.turbo_vision_on_key.clone()
        };
        let Some(handler) = handler else {
            return Ok(());
        };

        let key_event = turbo_vision_keyboard_to_console_key(event);
        let app_rec = Self::tui_application_record();
        let consumed = self.call_function_sync_allowing_shutdown(
            &handler,
            &[app_rec, Self::key_event_record(key_event)],
            line,
        )?;
        match consumed {
            fpas_bytecode::Value::Boolean(true) => event.clear(),
            fpas_bytecode::Value::Boolean(false) => {}
            other => {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("OnKey must return boolean, got {}", other.type_name()),
                    "Return `true` when the application consumed the key or `false` otherwise.",
                    line,
                ));
            }
        }
        Ok(())
    }

    fn dispatch_turbo_vision_unhandled_mouse(
        &mut self,
        event: &Event,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let handler = {
            let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
            tui.turbo_vision_on_mouse.clone()
        };
        let Some(handler) = handler else {
            return Ok(());
        };
        let Some(console_event) = turbo_vision_mouse_to_console_event(event) else {
            return Ok(());
        };
        let app_rec = Self::tui_application_record();
        let _ = self.call_function_sync_allowing_shutdown(
            &handler,
            &[app_rec, Self::console_event_record(console_event)],
            line,
        )?;
        Ok(())
    }

    /// Test hook for Turbo Vision keyboard conversion.
    #[cfg(test)]
    pub(crate) fn turbo_vision_keyboard_to_console_key_for_tests(event: &Event) -> ConsoleKeyEvent {
        turbo_vision_keyboard_to_console_key(event)
    }

    /// Test hook for unhandled Turbo Vision input dispatch.
    #[cfg(test)]
    pub(crate) fn dispatch_turbo_vision_unhandled_input_for_tests(
        &mut self,
        event: &mut Event,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        self.dispatch_turbo_vision_unhandled_input(event, line)
    }
}
