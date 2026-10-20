//! Server policy outcomes and typed lower-level failure causes.
use super::super::net::failure::Failure as NetFailure;
use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::native_errors::{self, NativeErrorFamily};

#[derive(Clone, Debug)]
pub(super) enum Failure {
    Argument(&'static str, String),
    State,
    Listener(u64, NetFailure),
    Signal { code: Option<i32>, detail: String },
    Io { code: Option<i32>, detail: String },
    Timeout,
    Invariant(VmError),
}
impl Failure {
    pub(super) fn signal(error: ctrlc::Error) -> Self {
        let (code, detail) = match &error {
            ctrlc::Error::System(source) => (source.raw_os_error(), format!("{error}: {source}")),
            _ => (None, error.to_string()),
        };
        Self::Signal { code, detail }
    }
    pub(super) fn io(error: std::io::Error) -> Self {
        Self::Io {
            code: error.raw_os_error(),
            detail: error.to_string(),
        }
    }
    pub(super) fn into_value(
        self,
        worker: &Worker,
        operation: &str,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        let (variant, fields) = match self {
            Self::Invariant(error) => return Err(error),
            Self::Argument(name, reason) => {
                (0, vec![Value::Str(name.into()), Value::Str(reason.into())])
            }
            Self::State => (1, vec![Value::Str(operation.into())]),
            Self::Listener(id, cause) => (
                2,
                vec![cause.into_value(
                    worker.layouts.as_ref(),
                    operation,
                    &[id.to_string()],
                    location,
                )?],
            ),
            Self::Signal { code, detail } => (
                3,
                vec![native_errors::host(
                    worker.layouts.as_ref(),
                    operation,
                    &[],
                    code,
                    detail,
                    location,
                )?],
            ),
            Self::Timeout => (4, vec![Value::Str(operation.into())]),
            Self::Io { code, detail } => (
                5,
                vec![native_errors::host(
                    worker.layouts.as_ref(),
                    operation,
                    &[],
                    code,
                    detail,
                    location,
                )?],
            ),
        };
        native_errors::value(
            worker.layouts.as_ref(),
            NativeErrorFamily::Server,
            variant,
            fields,
            location,
        )
    }
}
