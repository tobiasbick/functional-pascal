use super::super::super::{Vm, VmError};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};

impl Vm {
    pub(super) fn try_exec_console_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::ConsoleReadLn => {
                let text = self.text_input.read_line(line)?;
                self.push(Value::Str(text))?;
            }
            Intrinsic::ConsoleRead => {
                let ch = self.text_input.read_char(line)?;
                self.push(Value::Char(ch))?;
            }
            Intrinsic::ConsoleReadKey => {
                let ch = self.key_input.read_key(line)?;
                self.push(Value::Char(ch))?;
            }
            Intrinsic::ConsoleKeyPressed => {
                let pressed = self.key_input.key_pressed(line)?;
                self.push(Value::Boolean(pressed))?;
            }
            Intrinsic::ConsoleReadKeyEvent => {
                let event = self.key_input.read_key_event(line)?;
                self.push(self.key_event_record(event))?;
            }
            Intrinsic::ConsoleEventPending => {
                let pending = self.key_input.event_pending(line)?;
                self.push(Value::Boolean(pending))?;
            }
            Intrinsic::ConsoleReadEvent => {
                let event = self.key_input.read_event(line)?;
                if event.kind == fpas_std::event_kind_index("Resize") {
                    self.console.resize(event.width as u16, event.height as u16);
                }
                self.push(self.console_event_record(event))?;
            }
            Intrinsic::ConsoleClrScr => {
                self.console.clr_scr(line)?;
            }
            Intrinsic::ConsoleClrEol => {
                self.console.clr_eol(line)?;
            }
            Intrinsic::ConsoleGotoXY => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.console.goto_xy(x, y, line)?;
            }
            Intrinsic::ConsoleWhereX => {
                self.push(Value::Integer(self.console.where_x()))?;
            }
            Intrinsic::ConsoleWhereY => {
                self.push(Value::Integer(self.console.where_y()))?;
            }
            Intrinsic::ConsoleWindMin => {
                self.push(Value::Integer(self.console.wind_min()))?;
            }
            Intrinsic::ConsoleWindMax => {
                self.push(Value::Integer(self.console.wind_max()))?;
            }
            Intrinsic::ConsoleDelLine => {
                self.console.del_line(line)?;
            }
            Intrinsic::ConsoleInsLine => {
                self.console.ins_line(line)?;
            }
            Intrinsic::ConsoleWindow => {
                let y2 = self.pop_int(line)?;
                let x2 = self.pop_int(line)?;
                let y1 = self.pop_int(line)?;
                let x1 = self.pop_int(line)?;
                self.console.window(x1, y1, x2, y2, line)?;
            }
            Intrinsic::ConsoleTextColor => {
                let color = self.pop_int(line)?;
                self.console.text_color(color, line)?;
            }
            Intrinsic::ConsoleTextBackground => {
                let color = self.pop_int(line)?;
                self.console.text_background(color, line)?;
            }
            Intrinsic::ConsoleHighVideo => {
                self.console.high_video(line)?;
            }
            Intrinsic::ConsoleLowVideo => {
                self.console.low_video(line)?;
            }
            Intrinsic::ConsoleNormVideo => {
                self.console.norm_video(line)?;
            }
            Intrinsic::ConsoleTextAttr => {
                self.push(Value::Integer(self.console.text_attr()))?;
            }
            Intrinsic::ConsoleSetTextAttr => {
                let attr = self.pop_int(line)?;
                self.console.set_text_attr(attr, line)?;
            }
            Intrinsic::ConsoleDelay => {
                let ms = self.pop_int(line)?;
                self.console.delay(ms, line)?;
            }
            Intrinsic::ConsoleCursorOn => {
                self.console.cursor_on(line)?;
            }
            Intrinsic::ConsoleCursorOff => {
                self.console.cursor_off(line)?;
            }
            Intrinsic::ConsoleCursorBig => {
                self.console.cursor_big(line)?;
            }
            Intrinsic::ConsoleTextMode => {
                let mode = self.pop_int(line)?;
                self.console.text_mode(mode, line)?;
            }
            Intrinsic::ConsoleLastMode => {
                self.push(Value::Integer(self.console.last_mode()))?;
            }
            Intrinsic::ConsoleScreenWidth => {
                self.push(Value::Integer(self.console.screen_width()))?;
            }
            Intrinsic::ConsoleScreenHeight => {
                self.push(Value::Integer(self.console.screen_height()))?;
            }
            Intrinsic::ConsoleSound => {
                let hz = self.pop_int(line)?;
                self.console.sound(hz, line)?;
            }
            Intrinsic::ConsoleNoSound => {
                self.console.no_sound()?;
            }
            Intrinsic::ConsoleAssignCrt => {
                self.console.assign_crt()?;
            }
            Intrinsic::ConsoleEnableRawMode => {
                self.key_input.enable_raw_mode_explicit(line)?;
            }
            Intrinsic::ConsoleDisableRawMode => {
                self.key_input.disable_raw_mode_explicit(line)?;
            }
            Intrinsic::ConsoleEnterAltScreen => {
                self.console.enter_alt_screen(line)?;
            }
            Intrinsic::ConsoleLeaveAltScreen => {
                self.console.leave_alt_screen(line)?;
            }
            Intrinsic::ConsoleEnableMouse => {
                self.console.enable_mouse(line)?;
            }
            Intrinsic::ConsoleDisableMouse => {
                self.console.disable_mouse(line)?;
            }
            Intrinsic::ConsoleEnableFocus => {
                self.console.enable_focus(line)?;
            }
            Intrinsic::ConsoleDisableFocus => {
                self.console.disable_focus(line)?;
            }
            Intrinsic::ConsoleEnablePaste => {
                self.console.enable_paste(line)?;
            }
            Intrinsic::ConsoleDisablePaste => {
                self.console.disable_paste(line)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn key_event_record(&self, event: fpas_std::ConsoleKeyEvent) -> Value {
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

    fn console_event_record(&self, event: fpas_std::ConsoleEvent) -> Value {
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
                ("key".into(), self.key_event_record(key)),
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
