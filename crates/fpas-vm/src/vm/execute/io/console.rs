use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_std::{Console, KeyInput, TextInput};

impl Worker {
    pub(in crate::vm::execute) fn with_console<R>(&self, f: impl FnOnce(&mut Console) -> R) -> R {
        f(&mut self
            .shared
            .console
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }

    pub(in crate::vm::execute) fn with_console_and_key_input<R>(
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

    pub(in crate::vm::execute) fn with_key_input<R>(
        &self,
        f: impl FnOnce(&mut KeyInput) -> R,
    ) -> R {
        f(&mut self
            .shared
            .key_input
            .lock()
            .unwrap_or_else(|e| e.into_inner()))
    }

    pub(in crate::vm::execute::io) fn with_text_input<R>(
        &self,
        f: impl FnOnce(&mut TextInput) -> R,
    ) -> R {
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

    /// Executes one hosted `Std.Console` intrinsic when recognized.
    pub(super) fn try_exec_console_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Console(ConsoleIntrinsic::ReadLn) => {
                let text = self.with_text_input(|t| t.read_line(line))?;
                self.push(Value::Str(text))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::Read) => {
                let ch = self.with_text_input(|t| t.read_char(line))?;
                self.push(Value::Str(ch.to_string()))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::ReadKey) => {
                let ch = self.with_key_input(|k| k.read_key(line))?;
                self.push(Value::Str(ch.to_string()))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::KeyPressed) => {
                let pressed = self.with_key_input(|k| k.key_pressed(line))?;
                self.push(Value::Boolean(pressed))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::ReadKeyEvent) => {
                let event = self.with_key_input(|k| k.read_key_event(line))?;
                self.push(Self::key_event_record(event))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::EventPending) => {
                let pending = self.with_key_input(|k| k.event_pending(line))?;
                self.push(Value::Boolean(pending))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::ReadEvent) => {
                let event = self.with_key_input(|k| k.read_event(line))?;
                self.maybe_resize_on_event(&event);
                self.push(Self::console_event_record(event))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::ClrScr) => {
                self.with_console(|c| c.clr_scr(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::ClrEol) => {
                self.with_console(|c| c.clr_eol(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::GotoXY) => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.with_console(|c| c.goto_xy(x, y, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::WhereX) => {
                let val = self.with_console(|c| c.where_x());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::WhereY) => {
                let val = self.with_console(|c| c.where_y());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::WindMin) => {
                let val = self.with_console(|c| c.wind_min());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::WindMax) => {
                let val = self.with_console(|c| c.wind_max());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::DelLine) => {
                self.with_console(|c| c.del_line(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::InsLine) => {
                self.with_console(|c| c.ins_line(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::Window) => {
                let y2 = self.pop_int(line)?;
                let x2 = self.pop_int(line)?;
                let y1 = self.pop_int(line)?;
                let x1 = self.pop_int(line)?;
                self.with_console(|c| c.window(x1, y1, x2, y2, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::TextColor) => {
                let color = self.pop_int(line)?;
                self.with_console(|c| c.text_color(color, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::TextBackground) => {
                let color = self.pop_int(line)?;
                self.with_console(|c| c.text_background(color, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::TextColorRGB) => {
                let b = self.pop_int(line)?;
                let g = self.pop_int(line)?;
                let r = self.pop_int(line)?;
                self.with_console(|c| c.text_color_rgb(r, g, b, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::TextBackgroundRGB) => {
                let b = self.pop_int(line)?;
                let g = self.pop_int(line)?;
                let r = self.pop_int(line)?;
                self.with_console(|c| c.text_background_rgb(r, g, b, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::TextColor256) => {
                let index = self.pop_int(line)?;
                self.with_console(|c| c.text_color_256(index, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::TextBackground256) => {
                let index = self.pop_int(line)?;
                self.with_console(|c| c.text_background_256(index, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::HighVideo) => {
                self.with_console(|c| c.high_video(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::LowVideo) => {
                self.with_console(|c| c.low_video(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::NormVideo) => {
                self.with_console(|c| c.norm_video(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::TextAttr) => {
                let val = self.with_console(|c| c.text_attr());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::SetTextAttr) => {
                let attr = self.pop_int(line)?;
                self.with_console(|c| c.set_text_attr(attr, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::Delay) => {
                let ms = self.pop_int(line)?;
                self.with_console(|c| c.delay(ms, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::CursorOn) => {
                self.with_console(|c| c.cursor_on(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::CursorOff) => {
                self.with_console(|c| c.cursor_off(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::CursorBig) => {
                self.with_console(|c| c.cursor_big(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::TextMode) => {
                let mode = self.pop_int(line)?;
                self.with_console(|c| c.text_mode(mode, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::LastMode) => {
                let val = self.with_console(|c| c.last_mode());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::ScreenWidth) => {
                let val = self.with_console(|c| c.screen_width());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::ScreenHeight) => {
                let val = self.with_console(|c| c.screen_height());
                self.push(Value::Integer(val))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::Sound) => {
                let hz = self.pop_int(line)?;
                self.with_console(|c| c.sound(hz, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::NoSound) => self.with_console(|c| c.no_sound())?,
            Intrinsic::Console(ConsoleIntrinsic::AssignCrt) => {
                self.with_console(|c| c.assign_crt())?
            }
            Intrinsic::Console(ConsoleIntrinsic::EnableRawMode) => {
                self.with_key_input(|k| k.enable_raw_mode_explicit(line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::DisableRawMode) => {
                self.with_key_input(|k| k.disable_raw_mode_explicit(line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::EnterAltScreen) => {
                self.with_console(|c| c.enter_alt_screen(line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::LeaveAltScreen) => {
                self.with_console(|c| c.leave_alt_screen(line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::EnableMouse) => {
                self.with_console(|c| c.enable_mouse(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::DisableMouse) => {
                self.with_console(|c| c.disable_mouse(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::EnableFocus) => {
                self.with_console(|c| c.enable_focus(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::DisableFocus) => {
                self.with_console(|c| c.disable_focus(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::EnablePaste) => {
                self.with_console(|c| c.enable_paste(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::DisablePaste) => {
                self.with_console(|c| c.disable_paste(line))?
            }
            Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout) => {
                let ms = self.pop_int(line)?;
                let event = self.with_key_input(|k| k.read_event_timeout(ms, line))?;
                self.push_optional_event(event)?;
            }
            Intrinsic::Console(ConsoleIntrinsic::PollEvent) => {
                let event = self.with_key_input(|k| k.poll_event(line))?;
                self.push_optional_event(event)?;
            }
            Intrinsic::Console(ConsoleIntrinsic::CrtColor) => {
                let index = self.pop_int(line)?;
                let color = Self::console_crt_color(index, line)?;
                self.push(Self::console_color_record(color))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::Ansi256Color) => {
                let index = self.pop_int(line)?;
                let color = Self::console_ansi256_color(index, line)?;
                self.push(Self::console_color_record(color))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::RgbColor) => {
                let blue = self.pop_int(line)?;
                let green = self.pop_int(line)?;
                let red = self.pop_int(line)?;
                let color = Self::console_rgb_color(red, green, blue, line)?;
                self.push(Self::console_color_record(color))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::BeginFrame) => {
                self.with_console(|console| console.begin_frame());
            }
            Intrinsic::Console(ConsoleIntrinsic::Present) => {
                self.with_console(|console| console.present(line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::PutCell) => {
                let cell = self.pop_console_cell(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.with_console(|console| console.put_cell(x, y, cell, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::GetCell) => {
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                let cell = self.with_console(|console| console.get_cell(x, y));
                let value = cell
                    .map(|cell| Value::OptionSome(Box::new(Self::console_cell_record(cell))))
                    .unwrap_or(Value::OptionNone);
                self.push(value)?;
            }
            Intrinsic::Console(ConsoleIntrinsic::FillRect) => {
                let cell = self.pop_console_cell(line)?;
                let rect = self.pop_console_rect(line)?;
                self.with_console(|console| console.fill_rect(rect, cell, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::WriteCells) => {
                let cells = self.pop_console_cells(line)?;
                let y = self.pop_int(line)?;
                let x = self.pop_int(line)?;
                self.with_console(|console| console.write_cells(x, y, &cells, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::SaveRegion) => {
                let rect = self.pop_console_rect(line)?;
                let id = self.with_console(|console| console.save_region(rect, line))?;
                self.push(Self::saved_region_record(id))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::RestoreRegion) => {
                let id = self.pop_saved_region(line)?;
                self.with_console(|console| console.restore_region(id, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::DiscardRegion) => {
                let id = self.pop_saved_region(line)?;
                self.with_console(|console| console.discard_region(id, line))?;
            }
            Intrinsic::Console(ConsoleIntrinsic::DisplayWidth) => {
                let text = self.pop_console_text(line)?;
                self.push(Value::Integer(Console::display_width(&text)))?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
