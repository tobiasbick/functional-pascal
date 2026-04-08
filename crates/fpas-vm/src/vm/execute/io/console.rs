use super::super::super::Worker;
use super::super::super::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_std::{Console, KeyInput, TextInput};

impl Worker {
    pub(in super::super) fn with_console<R>(&self, f: impl FnOnce(&mut Console) -> R) -> R {
        f(&mut self
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }

    pub(in super::super) fn with_console_and_key_input<R>(
        &self,
        f: impl FnOnce(&mut Console, &mut KeyInput) -> R,
    ) -> R {
        let mut console = self
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut key_input = self
            .shared
            .key_input
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        f(&mut console, &mut key_input)
    }

    pub(in super::super) fn with_key_input<R>(&self, f: impl FnOnce(&mut KeyInput) -> R) -> R {
        f(&mut self
            .shared
            .key_input
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }

    fn with_text_input<R>(&self, f: impl FnOnce(&mut TextInput) -> R) -> R {
        f(&mut self
            .shared
            .text_input
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }

    /// If the event is a resize, update console dimensions.
    fn maybe_resize_on_event(&self, event: &fpas_std::ConsoleEvent) {
        if event.kind == fpas_std::event_kind_index("Resize") {
            self.with_console(|c| c.resize(event.width as u16, event.height as u16));
        }
    }

    /// Push a console event as `Option<Std.Console.Event>`.
    fn push_optional_event(
        &mut self,
        event: Option<fpas_std::ConsoleEvent>,
    ) -> Result<(), VmError> {
        match event {
            Some(ev) => {
                self.maybe_resize_on_event(&ev);
                self.push(Value::OptionSome(Box::new(Self::console_event_record(ev))))
            }
            None => self.push(Value::OptionNone),
        }
    }

    pub(super) fn try_exec_console_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::ConsoleReadLn => {
                let text = self.with_text_input(|t| t.read_line(line))?;
                self.push(Value::Str(text))?;
            }
            Intrinsic::ConsoleRead => {
                let ch = self.with_text_input(|t| t.read_char(line))?;
                self.push(Value::Char(ch))?;
            }
            Intrinsic::ConsoleReadKey => {
                let ch = self.with_key_input(|k| k.read_key(line))?;
                self.push(Value::Char(ch))?;
            }
            Intrinsic::ConsoleKeyPressed => {
                let pressed = self.with_key_input(|k| k.key_pressed(line))?;
                self.push(Value::Boolean(pressed))?;
            }
            Intrinsic::ConsoleReadKeyEvent => {
                let event = self.with_key_input(|k| k.read_key_event(line))?;
                self.push(Self::key_event_record(event))?;
            }
            Intrinsic::ConsoleEventPending => {
                let pending = self.with_key_input(|k| k.event_pending(line))?;
                self.push(Value::Boolean(pending))?;
            }
            Intrinsic::ConsoleReadEvent => {
                let event = self.with_key_input(|k| k.read_event(line))?;
                self.maybe_resize_on_event(&event);
                self.push(Self::console_event_record(event))?;
            }
            Intrinsic::ConsoleClrScr => self.with_console(|c| c.clr_scr(line))?,
            Intrinsic::ConsoleClrEol => self.with_console(|c| c.clr_eol(line))?,
            Intrinsic::ConsoleGotoXY => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.with_console(|c| c.goto_xy(x, y, line))?;
            }
            Intrinsic::ConsoleWhereX => {
                let val = self.with_console(|c| c.where_x());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleWhereY => {
                let val = self.with_console(|c| c.where_y());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleWindMin => {
                let val = self.with_console(|c| c.wind_min());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleWindMax => {
                let val = self.with_console(|c| c.wind_max());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleDelLine => self.with_console(|c| c.del_line(line))?,
            Intrinsic::ConsoleInsLine => self.with_console(|c| c.ins_line(line))?,
            Intrinsic::ConsoleWindow => {
                let y2 = self.pop_int(line)?;
                let x2 = self.pop_int(line)?;
                let y1 = self.pop_int(line)?;
                let x1 = self.pop_int(line)?;
                self.with_console(|c| c.window(x1, y1, x2, y2, line))?;
            }
            Intrinsic::ConsoleTextColor => {
                let color = self.pop_int(line)?;
                self.with_console(|c| c.text_color(color, line))?;
            }
            Intrinsic::ConsoleTextBackground => {
                let color = self.pop_int(line)?;
                self.with_console(|c| c.text_background(color, line))?;
            }
            Intrinsic::ConsoleTextColorRGB => {
                let b = self.pop_int(line)?;
                let g = self.pop_int(line)?;
                let r = self.pop_int(line)?;
                self.with_console(|c| c.text_color_rgb(r, g, b, line))?;
            }
            Intrinsic::ConsoleTextBackgroundRGB => {
                let b = self.pop_int(line)?;
                let g = self.pop_int(line)?;
                let r = self.pop_int(line)?;
                self.with_console(|c| c.text_background_rgb(r, g, b, line))?;
            }
            Intrinsic::ConsoleTextColor256 => {
                let index = self.pop_int(line)?;
                self.with_console(|c| c.text_color_256(index, line))?;
            }
            Intrinsic::ConsoleTextBackground256 => {
                let index = self.pop_int(line)?;
                self.with_console(|c| c.text_background_256(index, line))?;
            }
            Intrinsic::ConsoleHighVideo => self.with_console(|c| c.high_video(line))?,
            Intrinsic::ConsoleLowVideo => self.with_console(|c| c.low_video(line))?,
            Intrinsic::ConsoleNormVideo => self.with_console(|c| c.norm_video(line))?,
            Intrinsic::ConsoleTextAttr => {
                let val = self.with_console(|c| c.text_attr());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleSetTextAttr => {
                let attr = self.pop_int(line)?;
                self.with_console(|c| c.set_text_attr(attr, line))?;
            }
            Intrinsic::ConsoleDelay => {
                let ms = self.pop_int(line)?;
                self.with_console(|c| c.delay(ms, line))?;
            }
            Intrinsic::ConsoleCursorOn => self.with_console(|c| c.cursor_on(line))?,
            Intrinsic::ConsoleCursorOff => self.with_console(|c| c.cursor_off(line))?,
            Intrinsic::ConsoleCursorBig => self.with_console(|c| c.cursor_big(line))?,
            Intrinsic::ConsoleTextMode => {
                let mode = self.pop_int(line)?;
                self.with_console(|c| c.text_mode(mode, line))?;
            }
            Intrinsic::ConsoleLastMode => {
                let val = self.with_console(|c| c.last_mode());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleScreenWidth => {
                let val = self.with_console(|c| c.screen_width());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleScreenHeight => {
                let val = self.with_console(|c| c.screen_height());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleSound => {
                let hz = self.pop_int(line)?;
                self.with_console(|c| c.sound(hz, line))?;
            }
            Intrinsic::ConsoleNoSound => self.with_console(|c| c.no_sound())?,
            Intrinsic::ConsoleAssignCrt => self.with_console(|c| c.assign_crt())?,
            Intrinsic::ConsoleEnableRawMode => {
                self.with_key_input(|k| k.enable_raw_mode_explicit(line))?;
            }
            Intrinsic::ConsoleDisableRawMode => {
                self.with_key_input(|k| k.disable_raw_mode_explicit(line))?;
            }
            Intrinsic::ConsoleEnterAltScreen => {
                self.with_console(|c| c.enter_alt_screen(line))?;
            }
            Intrinsic::ConsoleLeaveAltScreen => {
                self.with_console(|c| c.leave_alt_screen(line))?;
            }
            Intrinsic::ConsoleEnableMouse => self.with_console(|c| c.enable_mouse(line))?,
            Intrinsic::ConsoleDisableMouse => self.with_console(|c| c.disable_mouse(line))?,
            Intrinsic::ConsoleEnableFocus => self.with_console(|c| c.enable_focus(line))?,
            Intrinsic::ConsoleDisableFocus => self.with_console(|c| c.disable_focus(line))?,
            Intrinsic::ConsoleEnablePaste => self.with_console(|c| c.enable_paste(line))?,
            Intrinsic::ConsoleDisablePaste => self.with_console(|c| c.disable_paste(line))?,
            Intrinsic::ConsoleReadEventTimeout => {
                let ms = self.pop_int(line)?;
                let event = self.with_key_input(|k| k.read_event_timeout(ms, line))?;
                self.push_optional_event(event)?;
            }
            Intrinsic::ConsolePollEvent => {
                let event = self.with_key_input(|k| k.poll_event(line))?;
                self.push_optional_event(event)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    pub(in crate::vm::execute::io) fn key_event_record(event: fpas_std::ConsoleKeyEvent) -> Value {
        Value::Record {
            type_name: "Std.Console.KeyEvent".into(),
            fields: vec![
                ("kind".into(), Value::Integer(event.kind as i64)),
                ("ch".into(), Value::Char(event.ch)),
                ("shift".into(), Value::Boolean(event.shift)),
                ("ctrl".into(), Value::Boolean(event.ctrl)),
                ("alt".into(), Value::Boolean(event.alt)),
                ("meta".into(), Value::Boolean(event.meta)),
            ],
        }
    }

    fn console_event_record(event: fpas_std::ConsoleEvent) -> Value {
        let fpas_std::ConsoleEvent {
            kind,
            key,
            mouse_action,
            mouse_button,
            mouse_x,
            mouse_y,
            width,
            height,
            text,
            shift,
            ctrl,
            alt,
            meta,
        } = event;
        Value::Record {
            type_name: "Std.Console.Event".into(),
            fields: vec![
                ("kind".into(), Value::Integer(kind as i64)),
                ("key".into(), Self::key_event_record(key)),
                ("mouse_action".into(), Value::Integer(mouse_action as i64)),
                ("mouse_button".into(), Value::Integer(mouse_button as i64)),
                ("mouse_x".into(), Value::Integer(mouse_x)),
                ("mouse_y".into(), Value::Integer(mouse_y)),
                ("width".into(), Value::Integer(width)),
                ("height".into(), Value::Integer(height)),
                ("text".into(), Value::Str(text)),
                ("shift".into(), Value::Boolean(shift)),
                ("ctrl".into(), Value::Boolean(ctrl)),
                ("alt".into(), Value::Boolean(alt)),
                ("meta".into(), Value::Boolean(meta)),
            ],
        }
    }
}
