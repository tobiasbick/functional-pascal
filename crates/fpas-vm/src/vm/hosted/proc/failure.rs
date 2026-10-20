//! Process failures preserve captured output after a started execution.

use std::io;

use fpas_bytecode::{SourceLocation, Value};
use fpas_std::native_errors::{self, NativeErrorFamily};

#[derive(Debug)]
pub(super) enum Kind {
    Argument(&'static str, String),
    Start(io::Error),
    Execution(io::Error),
    Wait(io::Error),
    Timeout,
    Cancelled,
    Limit(&'static str, usize),
}

#[derive(Debug)]
pub(super) struct Failure {
    pub kind: Kind,
    pub stdout: String,
    pub stderr: String,
}

impl Failure {
    pub(super) fn new(kind: Kind) -> Self {
        Self {
            kind,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub(super) fn argument(name: &'static str, reason: impl Into<String>) -> Self {
        Self::new(Kind::Argument(name, reason.into()))
    }

    pub(super) fn limit(resource: &'static str, limit: usize) -> Self {
        Self::new(Kind::Limit(resource, limit))
    }

    pub(super) fn into_value(
        self,
        factory: &dyn fpas_std::AggregateFactory,
        operation: &str,
        command: &str,
        legacy: bool,
        location: SourceLocation,
    ) -> Result<Value, fpas_std::StdError> {
        let output = vec![
            Value::Str(self.stdout.into()),
            Value::Str(self.stderr.into()),
        ];
        let cause = |error: io::Error| {
            native_errors::host(
                factory,
                operation,
                &[command.into()],
                error.raw_os_error(),
                error.to_string(),
                location,
            )
        };
        let (variant, fields) = match self.kind {
            Kind::Argument(name, reason) => {
                (2, vec![Value::Str(name.into()), Value::Str(reason.into())])
            }
            Kind::Start(error) => (0, vec![cause(error)?]),
            Kind::Wait(error) if legacy => (1, vec![cause(error)?]),
            Kind::Execution(error) if legacy => (3, vec![cause(error)?]),
            Kind::Execution(error) | Kind::Wait(error) => {
                let mut fields = vec![cause(error)?];
                fields.extend(output);
                (7, fields)
            }
            Kind::Timeout | Kind::Cancelled => {
                let variant = if matches!(self.kind, Kind::Timeout) {
                    4
                } else {
                    5
                };
                let mut fields = vec![Value::Str(operation.into())];
                fields.extend(output);
                (variant, fields)
            }
            Kind::Limit(resource, limit) => {
                let mut fields = vec![
                    Value::Str(operation.into()),
                    Value::Str(resource.into()),
                    Value::Integer(limit as i64),
                ];
                fields.extend(output);
                (6, fields)
            }
        };
        native_errors::value(factory, NativeErrorFamily::Proc, variant, fields, location)
    }
}
