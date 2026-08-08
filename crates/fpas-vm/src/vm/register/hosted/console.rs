//! Borrowed core `Std.Console` register intrinsics.

use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_std::Console;

use crate::vm::execute::io::console_cell_records::{
    console_ansi256_color, console_cell_from_value, console_cell_record, console_color_record,
    console_crt_color, console_rect_from_value, console_rgb_color, saved_region_from_value,
    saved_region_record,
};
use crate::vm::execute::io::console_records::{console_event_record, key_event_record};

use super::super::VmError;
use super::super::worker::RegisterWorker;
use super::console_args::{console_cells, integer, optional_event, require_count, string, value};

impl RegisterWorker {
    pub(super) fn execute_console_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Console(intrinsic) = intrinsic else {
            return Ok(None);
        };
        let result = match intrinsic {
            ConsoleIntrinsic::Write => {
                let value = value(arguments, 0, 1, self)?;
                self.with_console(|console| console.write(value, location))?;
                None
            }
            ConsoleIntrinsic::WriteLn => {
                let value = value(arguments, 0, 1, self)?;
                self.with_console(|console| console.write_ln(value, location))?;
                None
            }
            ConsoleIntrinsic::ReadLn => Some(Value::Str(
                self.with_text_input(|input| input.read_line(location))?
                    .into(),
            )),
            ConsoleIntrinsic::Read => Some(Value::Str(
                self.with_text_input(|input| input.read_char(location))?
                    .to_string()
                    .into(),
            )),
            ConsoleIntrinsic::ReadKey => Some(Value::Str(
                self.with_key_input(|input| input.read_key(location))?
                    .to_string()
                    .into(),
            )),
            ConsoleIntrinsic::KeyPressed => Some(Value::Boolean(
                self.with_key_input(|input| input.key_pressed(location))?,
            )),
            ConsoleIntrinsic::ReadKeyEvent => Some(key_event_record(
                self.with_key_input(|input| input.read_key_event(location))?,
            )),
            ConsoleIntrinsic::EventPending => Some(Value::Boolean(
                self.with_key_input(|input| input.event_pending(location))?,
            )),
            ConsoleIntrinsic::ReadEvent => Some(console_event_record(
                self.with_key_input(|input| input.read_event(location))?,
            )),
            ConsoleIntrinsic::ClrScr => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| console.clr_scr(location))?;
                None
            }
            ConsoleIntrinsic::ClrEol => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| console.clr_eol(location))?;
                None
            }
            ConsoleIntrinsic::GotoXY => {
                self.with_console(|console| {
                    console.goto_xy(
                        integer(arguments, 0, 2, self)?,
                        integer(arguments, 1, 2, self)?,
                        location,
                    )
                })?;
                None
            }
            ConsoleIntrinsic::WhereX => Some(Value::Integer(self.with_console(|c| c.where_x()))),
            ConsoleIntrinsic::WhereY => Some(Value::Integer(self.with_console(|c| c.where_y()))),
            ConsoleIntrinsic::WindMin => Some(Value::Integer(self.with_console(|c| c.wind_min()))),
            ConsoleIntrinsic::WindMax => Some(Value::Integer(self.with_console(|c| c.wind_max()))),
            ConsoleIntrinsic::DelLine => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| console.del_line(location))?;
                None
            }
            ConsoleIntrinsic::InsLine => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| console.ins_line(location))?;
                None
            }
            ConsoleIntrinsic::Window => {
                self.with_console(|console| {
                    console.window(
                        integer(arguments, 0, 4, self)?,
                        integer(arguments, 1, 4, self)?,
                        integer(arguments, 2, 4, self)?,
                        integer(arguments, 3, 4, self)?,
                        location,
                    )
                })?;
                None
            }
            ConsoleIntrinsic::TextColor => {
                let color = integer(arguments, 0, 1, self)?;
                self.with_console(|console| console.text_color(color, location))?;
                None
            }
            ConsoleIntrinsic::TextBackground => {
                let color = integer(arguments, 0, 1, self)?;
                self.with_console(|console| console.text_background(color, location))?;
                None
            }
            ConsoleIntrinsic::TextColorRGB | ConsoleIntrinsic::TextBackgroundRGB => {
                let red = integer(arguments, 0, 3, self)?;
                let green = integer(arguments, 1, 3, self)?;
                let blue = integer(arguments, 2, 3, self)?;
                if intrinsic == ConsoleIntrinsic::TextColorRGB {
                    self.with_console(|c| c.text_color_rgb(red, green, blue, location))?;
                } else {
                    self.with_console(|c| c.text_background_rgb(red, green, blue, location))?;
                }
                None
            }
            ConsoleIntrinsic::TextColor256 | ConsoleIntrinsic::TextBackground256 => {
                let index = integer(arguments, 0, 1, self)?;
                if intrinsic == ConsoleIntrinsic::TextColor256 {
                    self.with_console(|c| c.text_color_256(index, location))?;
                } else {
                    self.with_console(|c| c.text_background_256(index, location))?;
                }
                None
            }
            ConsoleIntrinsic::HighVideo => {
                require_count(arguments, 0, self)?;
                self.with_console(|c| c.high_video(location))?;
                None
            }
            ConsoleIntrinsic::LowVideo => {
                require_count(arguments, 0, self)?;
                self.with_console(|c| c.low_video(location))?;
                None
            }
            ConsoleIntrinsic::NormVideo => {
                require_count(arguments, 0, self)?;
                self.with_console(|c| c.norm_video(location))?;
                None
            }
            ConsoleIntrinsic::TextAttr => {
                Some(Value::Integer(self.with_console(|c| c.text_attr())))
            }
            ConsoleIntrinsic::SetTextAttr => {
                let attr = integer(arguments, 0, 1, self)?;
                self.with_console(|c| c.set_text_attr(attr, location))?;
                None
            }
            ConsoleIntrinsic::Delay => {
                Console::delay(integer(arguments, 0, 1, self)?, location)?;
                None
            }
            ConsoleIntrinsic::CursorOn
            | ConsoleIntrinsic::CursorOff
            | ConsoleIntrinsic::CursorBig => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| match intrinsic {
                    ConsoleIntrinsic::CursorOn => console.cursor_on(location),
                    ConsoleIntrinsic::CursorOff => console.cursor_off(location),
                    _ => console.cursor_big(location),
                })?;
                None
            }
            ConsoleIntrinsic::TextMode => {
                let mode = integer(arguments, 0, 1, self)?;
                self.with_console(|c| c.text_mode(mode, location))?;
                None
            }
            ConsoleIntrinsic::LastMode => {
                Some(Value::Integer(self.with_console(|c| c.last_mode())))
            }
            ConsoleIntrinsic::ScreenWidth => {
                Some(Value::Integer(self.with_console(|c| c.screen_width())))
            }
            ConsoleIntrinsic::ScreenHeight => {
                Some(Value::Integer(self.with_console(|c| c.screen_height())))
            }
            ConsoleIntrinsic::Sound => {
                let hertz = integer(arguments, 0, 1, self)?;
                self.with_console(|c| c.sound(hertz, location))?;
                None
            }
            ConsoleIntrinsic::NoSound => {
                require_count(arguments, 0, self)?;
                self.with_console(|c| c.no_sound())?;
                None
            }
            ConsoleIntrinsic::AssignCrt => {
                require_count(arguments, 0, self)?;
                self.with_console(|c| c.assign_crt())?;
                None
            }
            ConsoleIntrinsic::EnableRawMode | ConsoleIntrinsic::DisableRawMode => {
                require_count(arguments, 0, self)?;
                self.with_key_input(|input| {
                    if intrinsic == ConsoleIntrinsic::EnableRawMode {
                        input.enable_raw_mode_explicit(location)
                    } else {
                        input.disable_raw_mode_explicit(location)
                    }
                })?;
                None
            }
            ConsoleIntrinsic::EnterAltScreen
            | ConsoleIntrinsic::LeaveAltScreen
            | ConsoleIntrinsic::EnableMouse
            | ConsoleIntrinsic::DisableMouse
            | ConsoleIntrinsic::EnableFocus
            | ConsoleIntrinsic::DisableFocus
            | ConsoleIntrinsic::EnablePaste
            | ConsoleIntrinsic::DisablePaste => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| match intrinsic {
                    ConsoleIntrinsic::EnterAltScreen => console.enter_alt_screen(location),
                    ConsoleIntrinsic::LeaveAltScreen => console.leave_alt_screen(location),
                    ConsoleIntrinsic::EnableMouse => console.enable_mouse(location),
                    ConsoleIntrinsic::DisableMouse => console.disable_mouse(location),
                    ConsoleIntrinsic::EnableFocus => console.enable_focus(location),
                    ConsoleIntrinsic::DisableFocus => console.disable_focus(location),
                    ConsoleIntrinsic::EnablePaste => console.enable_paste(location),
                    _ => console.disable_paste(location),
                })?;
                None
            }
            ConsoleIntrinsic::AcquireInteractiveTerminal
            | ConsoleIntrinsic::ReleaseInteractiveTerminal => {
                require_count(arguments, 0, self)?;
                let mut console = self
                    .hosted
                    .console
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut input = self
                    .hosted
                    .key_input
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if intrinsic == ConsoleIntrinsic::AcquireInteractiveTerminal {
                    console.acquire_interactive_terminal(&mut input, location)?;
                } else {
                    console.release_interactive_terminal(&mut input, location)?;
                }
                None
            }
            ConsoleIntrinsic::ReadEventTimeout => {
                let milliseconds = integer(arguments, 0, 1, self)?;
                let event =
                    self.with_key_input(|input| input.read_event_timeout(milliseconds, location))?;
                Some(optional_event(event))
            }
            ConsoleIntrinsic::PollEvent => {
                require_count(arguments, 0, self)?;
                let event = self.with_key_input(|input| input.poll_event(location))?;
                Some(optional_event(event))
            }
            ConsoleIntrinsic::CrtColor => Some(console_color_record(console_crt_color(
                integer(arguments, 0, 1, self)?,
                location,
            )?)),
            ConsoleIntrinsic::Ansi256Color => Some(console_color_record(console_ansi256_color(
                integer(arguments, 0, 1, self)?,
                location,
            )?)),
            ConsoleIntrinsic::RgbColor => Some(console_color_record(console_rgb_color(
                integer(arguments, 0, 3, self)?,
                integer(arguments, 1, 3, self)?,
                integer(arguments, 2, 3, self)?,
                location,
            )?)),
            ConsoleIntrinsic::BeginFrame => {
                require_count(arguments, 0, self)?;
                self.with_console(Console::begin_frame);
                None
            }
            ConsoleIntrinsic::Present => {
                require_count(arguments, 0, self)?;
                self.with_console(|console| console.present(location))?;
                None
            }
            ConsoleIntrinsic::PutCell => {
                let cell = console_cell_from_value(value(arguments, 2, 3, self)?, location)?;
                self.with_console(|console| {
                    console.put_cell(
                        integer(arguments, 0, 3, self)?,
                        integer(arguments, 1, 3, self)?,
                        cell,
                        location,
                    )
                })?;
                None
            }
            ConsoleIntrinsic::GetCell => {
                let x = integer(arguments, 0, 2, self)?;
                let y = integer(arguments, 1, 2, self)?;
                let cell = self.with_console(|console| console.get_cell(x, y));
                Some(cell.map_or(Value::OptionNone, |cell| {
                    Value::OptionSome(Box::new(console_cell_record(cell)))
                }))
            }
            ConsoleIntrinsic::FillRect => {
                let rect = console_rect_from_value(value(arguments, 0, 2, self)?, location)?;
                let cell = console_cell_from_value(value(arguments, 1, 2, self)?, location)?;
                self.with_console(|console| console.fill_rect(rect, cell, location))?;
                None
            }
            ConsoleIntrinsic::WriteCells => {
                let cells = console_cells(value(arguments, 2, 3, self)?, location, self)?;
                self.with_console(|console| {
                    console.write_cells(
                        integer(arguments, 0, 3, self)?,
                        integer(arguments, 1, 3, self)?,
                        &cells,
                        location,
                    )
                })?;
                None
            }
            ConsoleIntrinsic::SaveRegion => {
                let rect = console_rect_from_value(value(arguments, 0, 1, self)?, location)?;
                let saved = self.with_console(|console| console.save_region(rect, location))?;
                Some(saved_region_record(saved))
            }
            ConsoleIntrinsic::RestoreRegion | ConsoleIntrinsic::DiscardRegion => {
                let saved = saved_region_from_value(value(arguments, 0, 1, self)?, location)?;
                if intrinsic == ConsoleIntrinsic::RestoreRegion {
                    self.with_console(|console| console.restore_region(saved, location))?;
                } else {
                    self.with_console(|console| console.discard_region(saved, location))?;
                }
                None
            }
            ConsoleIntrinsic::DisplayWidth | ConsoleIntrinsic::GraphemeWidth => {
                let text = string(arguments, 0, 1, self)?;
                let width = if intrinsic == ConsoleIntrinsic::DisplayWidth {
                    Console::display_width(text)
                } else {
                    Console::grapheme_width(text, location)?
                };
                Some(Value::Integer(width))
            }
            ConsoleIntrinsic::SplitGraphemes => {
                let text = string(arguments, 0, 1, self)?;
                Some(Value::Array(
                    Console::split_graphemes(text)
                        .into_iter()
                        .map(|grapheme| Value::Str(grapheme.into()))
                        .collect::<Vec<_>>()
                        .into(),
                ))
            }
        };
        Ok(Some(result))
    }

    pub(super) fn with_console<R>(&self, operation: impl FnOnce(&mut Console) -> R) -> R {
        operation(
            &mut self
                .hosted
                .console
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    pub(super) fn with_text_input<R>(
        &self,
        operation: impl FnOnce(&mut fpas_std::TextInput) -> R,
    ) -> R {
        operation(
            &mut self
                .hosted
                .text_input
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    fn with_key_input<R>(&self, operation: impl FnOnce(&mut fpas_std::KeyInput) -> R) -> R {
        operation(
            &mut self
                .hosted
                .key_input
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }
}
