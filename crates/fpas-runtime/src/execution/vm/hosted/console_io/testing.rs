//! Screen assertions and input helpers share console job and lock ownership.

use super::super::test_host as args;
use super::{lock, request::Output};
use crate::execution::vm::{VmError, hosted::HostedState, worker::Worker};
use crate::{SourceLocation, TestIntrinsic as Op, Value, stdlib::StdError};

pub(super) struct Call {
    location: SourceLocation,
    request: Request,
}
enum Request {
    Line(String, u16),
    Cell(u16, u16, char, i64, i64),
    Push(String),
    Scratch(std::path::PathBuf),
}

pub(super) fn supported(op: Op) -> bool {
    matches!(
        op,
        Op::AssertScreenLine | Op::AssertScreenCell | Op::PushReadLn | Op::ScratchDir
    )
}

impl Call {
    pub fn lanes(&self) -> [bool; 3] {
        match self.request {
            Request::Push(_) => [false, true, false],
            Request::Scratch(_) => [false; 3],
            _ => [true, false, false],
        }
    }
    pub fn location(&self) -> SourceLocation {
        self.location
    }
    pub fn prepare(
        worker: &Worker,
        op: Op,
        values: &[Value],
        location: SourceLocation,
    ) -> Result<Self, VmError> {
        let request = match op {
            Op::AssertScreenLine => Request::Line(
                args::string(values, 0, 2, worker)?.to_owned(),
                args::coordinate(args::integer(values, 1, 2, worker)?, "row", worker)?,
            ),
            Op::AssertScreenCell => {
                let x = args::coordinate(args::integer(values, 0, 5, worker)?, "column", worker)?;
                let y = args::coordinate(args::integer(values, 1, 5, worker)?, "row", worker)?;
                let mut chars = args::string(values, 2, 5, worker)?.chars();
                let glyph = match (chars.next(), chars.next()) {
                    (Some(c), None) => c,
                    _ => {
                        return Err(
                            worker.test_host_error("AssertScreenCell expected one character")
                        );
                    }
                };
                Request::Cell(
                    x,
                    y,
                    glyph,
                    args::integer(values, 3, 5, worker)?,
                    args::integer(values, 4, 5, worker)?,
                )
            }
            Op::PushReadLn => Request::Push(args::string(values, 0, 1, worker)?.to_owned()),
            Op::ScratchDir => {
                args::argument_count(values, 0, worker)?;
                Request::Scratch(
                    worker
                        .hosted
                        .test_scratch_dir
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .clone(),
                )
            }
            _ => unreachable!("hosted test operation"),
        };
        Ok(Self { location, request })
    }
    pub fn run(
        &self,
        state: &HostedState,
        cancelled: impl Fn() -> bool,
    ) -> Result<Output, StdError> {
        let location = self.location;
        if cancelled() {
            return Err(super::cancelled(location));
        }
        match &self.request {
            Request::Line(expected, row) => {
                let actual = lock(&state.console, &cancelled, location)?.query_screen_line(*row);
                crate::stdlib::assert_screen_line(expected.clone(), actual, location)?;
            }
            Request::Cell(x, y, glyph, fg, bg) => {
                let (actual, actual_fg, actual_bg) = lock(&state.console, &cancelled, location)?
                    .query_screen_cell(*x, *y)
                    .ok_or_else(|| {
                        error(
                            "AssertScreenCell coordinate is outside the screen",
                            location,
                        )
                    })?;
                crate::stdlib::assert_screen_cell(
                    *glyph, *fg, *bg, actual, actual_fg, actual_bg, location,
                )?;
            }
            Request::Push(text) => lock(&state.text_input, cancelled, location)?.push_line(text),
            Request::Scratch(path) => {
                std::fs::create_dir_all(path).map_err(|e| {
                    error(
                        &format!(
                            "Cannot create test scratch directory `{}`: {e}",
                            path.display()
                        ),
                        location,
                    )
                })?;
                return Ok(Output::Text(path.to_string_lossy().replace('\\', "/")));
            }
        }
        Ok(Output::Empty)
    }
}
fn error(message: &str, location: SourceLocation) -> StdError {
    crate::stdlib::error::std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_CONSOLE_STATE_ERROR,
        message,
        "Check the hosted console state and the assertion arguments.",
        location,
    )
}
