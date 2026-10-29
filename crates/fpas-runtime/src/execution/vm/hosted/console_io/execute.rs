//! The native side receives only owned text, cells, coordinates and host events.

use super::{
    lock,
    request::{Call, Output, Payload, Region},
};
use crate::execution::vm::hosted::HostedState;
use crate::{
    ConsoleIntrinsic as Op,
    stdlib::{Console, StdError},
};

impl Call {
    pub(super) fn run(
        &self,
        state: &HostedState,
        cancelled: impl Fn() -> bool,
    ) -> Result<Output, StdError> {
        let location = self.location;
        let [a, b, _, _] = self.numbers;
        match self.operation {
            Op::ReadLn => {
                return Ok(Output::OptionalText(
                    lock(&state.text_input, &cancelled, location)?.read_line(location)?,
                ));
            }
            Op::Read => {
                return Ok(Output::OptionalText(
                    lock(&state.text_input, &cancelled, location)?
                        .read_char(location)?
                        .map(|c| c.to_string()),
                ));
            }
            Op::ReadKey
            | Op::ReadKeyEvent
            | Op::ReadEvent
            | Op::ReadEventTimeout
            | Op::PollEvent
            | Op::KeyPressed
            | Op::EventPending
            | Op::EnableRawMode
            | Op::DisableRawMode => {
                let mut input = lock(&state.key_input, &cancelled, location)?;
                return Ok(match self.operation {
                    Op::ReadKey => {
                        Output::Text(input.read_key_controlled(location, &cancelled)?.to_string())
                    }
                    Op::ReadKeyEvent => {
                        Output::Key(input.read_key_event_controlled(location, &cancelled)?)
                    }
                    Op::ReadEvent => {
                        Output::Event(input.read_event_controlled(location, &cancelled)?)
                    }
                    Op::ReadEventTimeout => Output::OptionalEvent(
                        input.read_event_timeout_controlled(a, location, &cancelled)?,
                    ),
                    Op::PollEvent => Output::OptionalEvent(input.poll_event(location)?),
                    Op::KeyPressed => Output::Boolean(input.key_pressed(location)?),
                    Op::EventPending => Output::Boolean(input.event_pending(location)?),
                    Op::EnableRawMode => {
                        input.enable_raw_mode_explicit(location)?;
                        Output::Empty
                    }
                    Op::DisableRawMode => {
                        input.disable_raw_mode_explicit(location)?;
                        Output::Empty
                    }
                    _ => unreachable!("admitted input operation"),
                });
            }
            _ => {}
        }
        let mut console = lock(&state.console, &cancelled, location)?;
        super::discard_regions(state, &mut console);
        let out = match self.operation {
            Op::Write | Op::WriteLn => {
                let Payload::Text(text) = &self.payload else {
                    unreachable!("write payload")
                };
                if self.operation == Op::Write {
                    console.write_text(text, location)?;
                } else {
                    console.write_line_text(text, location)?;
                }
                Output::Empty
            }
            Op::WhereX => Output::Integer(console.where_x()),
            Op::WhereY => Output::Integer(console.where_y()),
            Op::WindMin => Output::Integer(console.wind_min()),
            Op::WindMax => Output::Integer(console.wind_max()),
            Op::TextAttr => Output::Integer(console.text_attr()),
            Op::LastMode => Output::Integer(console.last_mode()),
            Op::ScreenWidth => Output::Integer(console.screen_width()),
            Op::ScreenHeight => Output::Integer(console.screen_height()),
            Op::GetCell => Output::Cell(console.get_cell(a, b)),
            Op::SaveRegion => {
                let Payload::Rect(rect) = self.payload else {
                    unreachable!("region rectangle")
                };
                Output::Region(Region {
                    id: Some(console.save_region(rect, location)?),
                    cleanup: state.console_regions.clone(),
                })
            }
            _ => {
                self.change(&mut console, state, &cancelled)?;
                Output::Empty
            }
        };
        Ok(out)
    }

    fn change(
        &self,
        console: &mut Console,
        state: &HostedState,
        cancelled: impl Fn() -> bool,
    ) -> Result<(), StdError> {
        let location = self.location;
        let [a, b, c, d] = self.numbers;
        match self.operation {
            Op::ClrScr => console.clr_scr(location)?,
            Op::ClrEol => console.clr_eol(location)?,
            Op::GotoXY => console.goto_xy(a, b, location)?,
            Op::DelLine => console.del_line(location)?,
            Op::InsLine => console.ins_line(location)?,
            Op::Window => console.window(a, b, c, d, location)?,
            Op::TextColor => console.text_color(a, location)?,
            Op::TextBackground => console.text_background(a, location)?,
            Op::TextColorRGB => console.text_color_rgb(a, b, c, location)?,
            Op::TextBackgroundRGB => console.text_background_rgb(a, b, c, location)?,
            Op::TextColor256 => console.text_color_256(a, location)?,
            Op::TextBackground256 => console.text_background_256(a, location)?,
            Op::HighVideo => console.high_video(location)?,
            Op::LowVideo => console.low_video(location)?,
            Op::NormVideo => console.norm_video(location)?,
            Op::SetTextAttr => console.set_text_attr(a, location)?,
            Op::CursorOn => console.cursor_on(location)?,
            Op::CursorOff => console.cursor_off(location)?,
            Op::CursorBig => console.cursor_big(location)?,
            Op::TextMode => console.text_mode(a, location)?,
            Op::Sound => console.sound(a, location)?,
            Op::NoSound => console.no_sound()?,
            Op::AssignCrt => console.assign_crt()?,
            Op::EnterAltScreen => console.enter_alt_screen(location)?,
            Op::LeaveAltScreen => console.leave_alt_screen(location)?,
            Op::EnableMouse => console.enable_mouse(location)?,
            Op::DisableMouse => console.disable_mouse(location)?,
            Op::EnableFocus => console.enable_focus(location)?,
            Op::DisableFocus => console.disable_focus(location)?,
            Op::EnablePaste => console.enable_paste(location)?,
            Op::DisablePaste => console.disable_paste(location)?,
            Op::AcquireInteractiveTerminal | Op::ReleaseInteractiveTerminal => {
                let mut input = lock(&state.key_input, cancelled, location)?;
                if self.operation == Op::AcquireInteractiveTerminal {
                    console.acquire_interactive_terminal(&mut input, location)?;
                } else {
                    console.release_interactive_terminal(&mut input, location)?;
                }
            }
            Op::BeginFrame => console.begin_frame(),
            Op::Present => console.present(location)?,
            Op::PutCell => {
                let Payload::Cell(cell) = &self.payload else {
                    unreachable!("cell payload")
                };
                console.put_cell(a, b, cell.clone(), location)?;
            }
            Op::FillRect => {
                let Payload::RectCell(rect, cell) = &self.payload else {
                    unreachable!("rectangle payload")
                };
                console.fill_rect(*rect, cell.clone(), location)?;
            }
            Op::WriteCells => {
                let Payload::Cells(cells) = &self.payload else {
                    unreachable!("cells payload")
                };
                console.write_cells(a, b, cells, location)?;
            }
            Op::RestoreRegion | Op::DiscardRegion => {
                let Payload::Region(id) = self.payload else {
                    unreachable!("region payload")
                };
                if self.operation == Op::RestoreRegion {
                    console.restore_region(id, location)?;
                } else {
                    console.discard_region(id, location)?;
                }
            }
            _ => unreachable!("validated stateful console operation"),
        }
        Ok(())
    }
}
