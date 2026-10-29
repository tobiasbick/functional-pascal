//! Argument decoding and result allocation run exclusively on the owning VM task.

use super::super::{console_args as args, console_cell_records as cells};
use crate::execution::vm::{VmError, worker::Worker};
use crate::{
    ConsoleIntrinsic as Op, SourceLocation, Value,
    stdlib::{ConsoleCell, ConsoleRect, SavedRegionId},
};

pub(super) enum Payload {
    Empty,
    Text(String),
    Cell(ConsoleCell),
    Cells(Vec<ConsoleCell>),
    Rect(ConsoleRect),
    RectCell(ConsoleRect, ConsoleCell),
    Region(SavedRegionId),
}

pub(super) struct Call {
    pub operation: Op,
    pub numbers: [i64; 4],
    pub payload: Payload,
    pub location: SourceLocation,
}

pub(in crate::execution::vm) fn supported(op: Op) -> bool {
    !matches!(
        op,
        Op::Delay
            | Op::CrtColor
            | Op::Ansi256Color
            | Op::RgbColor
            | Op::DisplayWidth
            | Op::GraphemeWidth
            | Op::SplitGraphemes
    )
}

impl Call {
    pub(super) fn lanes(&self) -> [bool; 3] {
        match self.operation {
            Op::ReadLn | Op::Read => [false, true, false],
            Op::ReadKey
            | Op::ReadKeyEvent
            | Op::ReadEvent
            | Op::ReadEventTimeout
            | Op::PollEvent
            | Op::KeyPressed
            | Op::EventPending
            | Op::EnableRawMode
            | Op::DisableRawMode => [false, false, true],
            Op::AcquireInteractiveTerminal | Op::ReleaseInteractiveTerminal => [true, false, true],
            _ => [true, false, false],
        }
    }
    pub(super) fn prepare(
        worker: &Worker,
        operation: Op,
        values: &[Value],
        location: SourceLocation,
    ) -> Result<Self, VmError> {
        let mut call = Self {
            operation,
            numbers: [0; 4],
            payload: Payload::Empty,
            location,
        };
        let count = match operation {
            Op::Write | Op::WriteLn => {
                call.payload = Payload::Text(format!("{}", args::value(values, 0, 1, worker)?));
                return Ok(call);
            }
            Op::PutCell => {
                call.payload = Payload::Cell(cells::console_cell_from_value(
                    args::value(values, 2, 3, worker)?,
                    location,
                )?);
                call.numbers[0] = args::integer(values, 0, 3, worker)?;
                call.numbers[1] = args::integer(values, 1, 3, worker)?;
                return Ok(call);
            }
            Op::WriteCells => {
                call.payload = Payload::Cells(args::console_cells(
                    args::value(values, 2, 3, worker)?,
                    location,
                    worker,
                )?);
                call.numbers[0] = args::integer(values, 0, 3, worker)?;
                call.numbers[1] = args::integer(values, 1, 3, worker)?;
                return Ok(call);
            }
            Op::FillRect => {
                call.payload = Payload::RectCell(
                    cells::console_rect_from_value(args::value(values, 0, 2, worker)?, location)?,
                    cells::console_cell_from_value(args::value(values, 1, 2, worker)?, location)?,
                );
                return Ok(call);
            }
            Op::SaveRegion => {
                call.payload = Payload::Rect(cells::console_rect_from_value(
                    args::value(values, 0, 1, worker)?,
                    location,
                )?);
                return Ok(call);
            }
            Op::RestoreRegion | Op::DiscardRegion => {
                call.payload = Payload::Region(cells::saved_region_from_value(
                    args::value(values, 0, 1, worker)?,
                    location,
                )?);
                return Ok(call);
            }
            Op::Window => 4,
            Op::TextColorRGB | Op::TextBackgroundRGB => 3,
            Op::GotoXY | Op::GetCell => 2,
            Op::TextColor
            | Op::TextBackground
            | Op::TextColor256
            | Op::TextBackground256
            | Op::SetTextAttr
            | Op::TextMode
            | Op::Sound
            | Op::ReadEventTimeout => 1,
            _ => 0,
        };
        args::require_count(values, count, worker)?;
        for index in 0..count {
            call.numbers[index] = args::integer(values, index, count, worker)?;
        }
        Ok(call)
    }
}

pub(super) enum Output {
    Empty,
    Boolean(bool),
    Integer(i64),
    Text(String),
    OptionalText(Option<String>),
    Key(crate::stdlib::ConsoleKeyEvent),
    Event(crate::stdlib::ConsoleEvent),
    OptionalEvent(Option<crate::stdlib::ConsoleEvent>),
    Cell(Option<ConsoleCell>),
    Region(Region),
}

pub(super) struct Region {
    pub id: Option<SavedRegionId>,
    pub cleanup: std::sync::Arc<std::sync::Mutex<Vec<SavedRegionId>>>,
}

impl Drop for Region {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            self.cleanup
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .push(id);
        }
    }
}

impl Output {
    pub(super) fn into_value(
        self,
        worker: &Worker,
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        Ok(Some(match self {
            Self::Empty => return Ok(None),
            Self::Boolean(value) => Value::Boolean(value),
            Self::Integer(value) => Value::Integer(value),
            Self::Text(value) => Value::Str(value.into()),
            Self::OptionalText(value) => value.map_or(Value::OptionNone, |value| {
                Value::option_some(Value::Str(value.into()))
            }),
            Self::Key(value) => worker.key_event_record(value, location)?,
            Self::Event(value) => worker.console_event_record(value, location)?,
            Self::OptionalEvent(value) => args::optional_event(worker, value, location)?,
            Self::Cell(value) => match value {
                Some(value) => Value::option_some(worker.console_cell_record(value, location)?),
                None => Value::OptionNone,
            },
            Self::Region(mut value) => {
                let Some(id) = value.id.take() else {
                    unreachable!("region already published")
                };
                Value::OpaqueHandle(id.0)
            }
        }))
    }
}
