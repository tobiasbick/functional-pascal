//! Console value construction stays on the VM; stateful operations share owned native calls.

use super::{console_args as args, console_cell_records as cells, console_io};
use crate::execution::vm::{VmError, worker::Worker};
use crate::{ConsoleIntrinsic as Op, Intrinsic, SourceLocation, Value, stdlib::Console};

impl Worker {
    pub(super) fn execute_console_intrinsic(
        &self,
        intrinsic: Intrinsic,
        values: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Console(op) = intrinsic else {
            return Ok(None);
        };
        if console_io::supported(op) {
            return self.execute_console_owned(op, values, location).map(Some);
        }
        let value = match op {
            Op::Delay => {
                Console::delay(args::integer(values, 0, 1, self)?, location)?;
                return Ok(Some(None));
            }
            Op::CrtColor => cells::console_color_record(
                self,
                cells::console_crt_color(args::integer(values, 0, 1, self)?, location)?,
                location,
            )?,
            Op::Ansi256Color => cells::console_color_record(
                self,
                cells::console_ansi256_color(args::integer(values, 0, 1, self)?, location)?,
                location,
            )?,
            Op::RgbColor => cells::console_color_record(
                self,
                cells::console_rgb_color(
                    args::integer(values, 0, 3, self)?,
                    args::integer(values, 1, 3, self)?,
                    args::integer(values, 2, 3, self)?,
                    location,
                )?,
                location,
            )?,
            Op::DisplayWidth | Op::GraphemeWidth => {
                let text = args::string(values, 0, 1, self)?;
                Value::Integer(if op == Op::DisplayWidth {
                    Console::display_width(text)
                } else {
                    Console::grapheme_width(text, location)?
                })
            }
            Op::SplitGraphemes => Value::Array(
                Console::split_graphemes(args::string(values, 0, 1, self)?)
                    .into_iter()
                    .map(|s| Value::Str(s.into()))
                    .collect(),
            ),
            _ => unreachable!("stateful console operations were routed above"),
        };
        Ok(Some(Some(value)))
    }
}
