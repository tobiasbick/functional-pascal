use super::super::super::Worker;
use super::super::super::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, Value};

impl Worker {
    pub(super) fn try_exec_console_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::ConsoleReadLn => {
                let text = self
                    .shared
                    .text_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .read_line(line)?;
                self.push(Value::Str(text))?;
            }
            Intrinsic::ConsoleRead => {
                let ch = self
                    .shared
                    .text_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .read_char(line)?;
                self.push(Value::Char(ch))?;
            }
            Intrinsic::ConsoleReadKey => {
                let ch = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .read_key(line)?;
                self.push(Value::Char(ch))?;
            }
            Intrinsic::ConsoleKeyPressed => {
                let pressed = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .key_pressed(line)?;
                self.push(Value::Boolean(pressed))?;
            }
            Intrinsic::ConsoleReadKeyEvent => {
                let event = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .read_key_event(line)?;
                self.push(Self::key_event_record(event))?;
            }
            Intrinsic::ConsoleEventPending => {
                let pending = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .event_pending(line)?;
                self.push(Value::Boolean(pending))?;
            }
            Intrinsic::ConsoleReadEvent => {
                let event = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .read_event(line)?;
                {
                    let mut console = self
                        .shared
                        .console
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if event.kind == fpas_std::event_kind_index("Resize") {
                        console.resize(event.width as u16, event.height as u16);
                    }
                }
                self.push(Self::console_event_record(event))?;
            }
            Intrinsic::ConsoleClrScr => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clr_scr(line)?;
            }
            Intrinsic::ConsoleClrEol => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clr_eol(line)?;
            }
            Intrinsic::ConsoleGotoXY => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .goto_xy(x, y, line)?;
            }
            Intrinsic::ConsoleWhereX => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .where_x();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleWhereY => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .where_y();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleWindMin => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .wind_min();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleWindMax => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .wind_max();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleDelLine => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .del_line(line)?;
            }
            Intrinsic::ConsoleInsLine => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .ins_line(line)?;
            }
            Intrinsic::ConsoleWindow => {
                let y2 = self.pop_int(line)?;
                let x2 = self.pop_int(line)?;
                let y1 = self.pop_int(line)?;
                let x1 = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .window(x1, y1, x2, y2, line)?;
            }
            Intrinsic::ConsoleTextColor => {
                let color = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .text_color(color, line)?;
            }
            Intrinsic::ConsoleTextBackground => {
                let color = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .text_background(color, line)?;
            }
            Intrinsic::ConsoleHighVideo => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .high_video(line)?;
            }
            Intrinsic::ConsoleLowVideo => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .low_video(line)?;
            }
            Intrinsic::ConsoleNormVideo => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .norm_video(line)?;
            }
            Intrinsic::ConsoleTextAttr => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .text_attr();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleSetTextAttr => {
                let attr = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .set_text_attr(attr, line)?;
            }
            Intrinsic::ConsoleDelay => {
                let ms = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .delay(ms, line)?;
            }
            Intrinsic::ConsoleCursorOn => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .cursor_on(line)?;
            }
            Intrinsic::ConsoleCursorOff => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .cursor_off(line)?;
            }
            Intrinsic::ConsoleCursorBig => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .cursor_big(line)?;
            }
            Intrinsic::ConsoleTextMode => {
                let mode = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .text_mode(mode, line)?;
            }
            Intrinsic::ConsoleLastMode => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .last_mode();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleScreenWidth => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .screen_width();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleScreenHeight => {
                let val = self
                    .shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .screen_height();
                self.push(Value::Integer(val))?;
            }
            Intrinsic::ConsoleSound => {
                let hz = self.pop_int(line)?;
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .sound(hz, line)?;
            }
            Intrinsic::ConsoleNoSound => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .no_sound()?;
            }
            Intrinsic::ConsoleAssignCrt => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .assign_crt()?;
            }
            Intrinsic::ConsoleEnableRawMode => {
                self.shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .enable_raw_mode_explicit(line)?;
            }
            Intrinsic::ConsoleDisableRawMode => {
                self.shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .disable_raw_mode_explicit(line)?;
            }
            Intrinsic::ConsoleEnterAltScreen => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .enter_alt_screen(line)?;
            }
            Intrinsic::ConsoleLeaveAltScreen => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .leave_alt_screen(line)?;
            }
            Intrinsic::ConsoleEnableMouse => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .enable_mouse(line)?;
            }
            Intrinsic::ConsoleDisableMouse => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .disable_mouse(line)?;
            }
            Intrinsic::ConsoleEnableFocus => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .enable_focus(line)?;
            }
            Intrinsic::ConsoleDisableFocus => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .disable_focus(line)?;
            }
            Intrinsic::ConsoleEnablePaste => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .enable_paste(line)?;
            }
            Intrinsic::ConsoleDisablePaste => {
                self.shared
                    .console
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .disable_paste(line)?;
            }
            Intrinsic::ConsoleReadEventTimeout => {
                let ms = self.pop_int(line)?;
                let maybe_event = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .read_event_timeout(ms, line)?;
                match maybe_event {
                    Some(event) => {
                        {
                            let mut console = self
                                .shared
                                .console
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            if event.kind == fpas_std::event_kind_index("Resize") {
                                console.resize(event.width as u16, event.height as u16);
                            }
                        }
                        self.push(Value::OptionSome(Box::new(Self::console_event_record(event))))?;
                    }
                    None => {
                        self.push(Value::OptionNone)?;
                    }
                }
            }
            Intrinsic::ConsolePollEvent => {
                let maybe_event = self
                    .shared
                    .key_input
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .poll_event(line)?;
                match maybe_event {
                    Some(event) => {
                        {
                            let mut console = self
                                .shared
                                .console
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            if event.kind == fpas_std::event_kind_index("Resize") {
                                console.resize(event.width as u16, event.height as u16);
                            }
                        }
                        self.push(Value::OptionSome(Box::new(Self::console_event_record(event))))?;
                    }
                    None => {
                        self.push(Value::OptionNone)?;
                    }
                }
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn key_event_record(event: fpas_std::ConsoleKeyEvent) -> Value {
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
