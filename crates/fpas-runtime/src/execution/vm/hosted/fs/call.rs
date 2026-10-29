//! One native request shared by synchronous calls and suspended file operations.

use super::{arguments as args, failure::Failure, state::MAX_BYTES};
use crate::execution::vm::{VmError, hosted::HostedState, worker::Worker};
use crate::{FsIntrinsic as Op, Value};

enum Request {
    Path(crate::stdlib::fs::native::Call),
    Open(String, i64),
    Close(u64),
    Read(u64, i64),
    Write(u64, Vec<u8>),
    ReadLine(u64),
    Seek(u64, i64, i64),
    Tell(u64),
    Flush(u64),
    Size(u64),
    SetLength(u64, i64),
    WriteText(u64, String, bool),
}

pub(super) enum Output {
    Path(crate::stdlib::fs::native::Outcome),
    Handle(u64),
    Bytes(Vec<u8>),
    Number(i64),
    Line(Option<String>),
    Done,
}

pub(super) struct Call {
    request: Request,
    token: Option<u64>,
}

impl Call {
    pub(super) fn prepare(
        worker: &Worker,
        op: Op,
        values: &[Value],
    ) -> Result<Result<Self, Failure>, VmError> {
        if crate::stdlib::fs::native::supported(op) {
            let prepared = crate::stdlib::fs::native::Call::prepare_values(
                op,
                values,
                worker.layouts.as_ref(),
                worker.intrinsic_location(),
            )?;
            return Ok(prepared
                .map(|request| Self {
                    request: Request::Path(request),
                    token: None,
                })
                .map_err(Failure::Path));
        }
        let cancellable = matches!(
            op,
            Op::ReadWithCancellation
                | Op::WriteWithCancellation
                | Op::ReadLineWithCancellation
                | Op::WriteStringWithCancellation
                | Op::WriteLineWithCancellation
        );
        let count = match op {
            Op::Close
            | Op::Tell
            | Op::Flush
            | Op::Size
            | Op::ReadLine
            | Op::ReadLineWithCancellation => 1,
            Op::Seek => 3,
            _ => 2,
        };
        args::count(worker, values, count + usize::from(cancellable))?;
        let token = if cancellable {
            Some(args::handle(worker, &values[count], "Token")?)
        } else {
            None
        };
        let valid_token = token.map_or(Ok(false), |token| {
            worker
                .hosted
                .cancellations
                .is_cancelled(token)
                .map_err(|error| Failure::argument("Token", error))
        });
        let request = if op == Op::Open {
            let path = args::text(worker, &values[0], "Path")?;
            args::ordinal(&values[1], true).map(|mode| Request::Open(path.to_owned(), mode))
        } else {
            let handle = args::handle(worker, &values[0], "File")?;
            match op {
                Op::Close => Ok(Request::Close(handle)),
                Op::Tell => Ok(Request::Tell(handle)),
                Op::Flush => Ok(Request::Flush(handle)),
                Op::Size => Ok(Request::Size(handle)),
                Op::SetLength => Ok(Request::SetLength(
                    handle,
                    args::integer(worker, &values[1], "Length")?,
                )),
                Op::Seek => {
                    let offset = args::integer(worker, &values[1], "Offset")?;
                    args::ordinal(&values[2], false)
                        .map(|origin| Request::Seek(handle, offset, origin))
                }
                Op::Read | Op::ReadWithCancellation => Ok(Request::Read(
                    handle,
                    args::integer(worker, &values[1], "MaxBytes")?,
                )),
                Op::Write | Op::WriteWithCancellation => {
                    args::bytes(worker, &values[1])?.map(|bytes| Request::Write(handle, bytes))
                }
                Op::ReadLine | Op::ReadLineWithCancellation => Ok(Request::ReadLine(handle)),
                Op::WriteString
                | Op::WriteLine
                | Op::WriteStringWithCancellation
                | Op::WriteLineWithCancellation => {
                    let text = args::text(worker, &values[1], "Text")?;
                    if text.len() > MAX_BYTES {
                        return Ok(valid_token.and(Err(Failure::Limit(MAX_BYTES as i64))));
                    } else {
                        Ok(Request::WriteText(
                            handle,
                            text.to_owned(),
                            matches!(op, Op::WriteLine | Op::WriteLineWithCancellation),
                        ))
                    }
                }
                _ => unreachable!("only handle operations belong to hosted Fs dispatch"),
            }
        };
        Ok(request.and_then(|request| {
            valid_token?;
            match request {
                Request::Read(_, count) if count <= 0 => {
                    return Err(Failure::argument("MaxBytes", "Read size must be positive"));
                }
                Request::Read(_, count) if count > MAX_BYTES as i64 => {
                    return Err(Failure::Limit(MAX_BYTES as i64));
                }
                Request::SetLength(_, length) if length < 0 => {
                    return Err(Failure::argument(
                        "Length",
                        "File length must not be negative",
                    ));
                }
                Request::Seek(_, offset, 0) if offset < 0 => {
                    return Err(Failure::argument(
                        "Offset",
                        "Start offset must not be negative",
                    ));
                }
                _ => {}
            }
            Ok(Self { request, token })
        }))
    }

    pub(super) fn run(
        &self,
        hosted: &HostedState,
        mut stopped: impl FnMut() -> bool,
    ) -> Result<Output, Failure> {
        let files = &hosted.files;
        let path = matches!(&self.request, Request::Path(_));
        let cancelled = || {
            stopped()
                || (!path && files.is_stopping())
                || self
                    .token
                    .is_some_and(|id| hosted.cancellations.is_cancelled(id).unwrap_or(true))
        };
        Ok(match &self.request {
            Request::Path(request) => Output::Path(request.run(cancelled)),
            Request::Open(path, mode) => Output::Handle(files.open(path, *mode, cancelled)?),
            Request::Close(handle) => {
                files.close(*handle, cancelled)?;
                Output::Done
            }
            Request::Read(handle, count) => Output::Bytes(files.read(*handle, *count, cancelled)?),
            Request::Write(handle, bytes) => {
                Output::Number(files.write(*handle, bytes, cancelled)? as i64)
            }
            Request::ReadLine(handle) => Output::Line(files.read_line(*handle, cancelled)?),
            Request::Seek(handle, offset, origin) => {
                Output::Number(files.seek(*handle, *offset, *origin, cancelled)?)
            }
            Request::Tell(handle) => Output::Number(files.tell(*handle, cancelled)?),
            Request::Flush(handle) => {
                files.flush(*handle, cancelled)?;
                Output::Done
            }
            Request::Size(handle) => Output::Number(files.size(*handle, cancelled)?),
            Request::SetLength(handle, length) => {
                files.set_length(*handle, *length, cancelled)?;
                Output::Done
            }
            Request::WriteText(handle, text, newline) => {
                files.write_text(*handle, text, *newline, cancelled)?;
                Output::Done
            }
        })
    }
}
