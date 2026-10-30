//! Typed filesystem failures for VM-owned file handles.

use std::io;

use crate::stdlib::native_errors::{self, NativeErrorFamily};
use crate::{SourceLocation, Value};

#[derive(Debug)]
pub(super) enum Failure {
    Path(crate::stdlib::fs::failure::Failure),
    Closed,
    Cancelled,
    Argument(&'static str, String),
    Limit(i64),
    Encoding(String, String),
    Io(String, io::Error),
}

impl Failure {
    pub(super) fn argument(name: &'static str, reason: impl Into<String>) -> Self {
        Self::Argument(name, reason.into())
    }

    pub(super) fn io(path: &str, error: io::Error) -> Self {
        Self::Io(path.to_owned(), error)
    }

    pub(super) fn into_value(
        self,
        factory: &dyn crate::stdlib::AggregateFactory,
        operation: &str,
        location: SourceLocation,
    ) -> Result<Value, crate::stdlib::StdError> {
        if let Self::Path(error) = self {
            return crate::stdlib::fs::results::error_value(
                factory,
                operation,
                &[],
                error,
                location,
            );
        }
        let text = |value: &str| Value::Str(value.into());
        let (variant, fields) = match self {
            Self::Path(_) => unreachable!("path errors are converted above"),
            Self::Closed => (12, vec![text(operation)]),
            Self::Cancelled => (13, vec![text(operation)]),
            Self::Argument(name, reason) => (8, vec![text(name), text(&reason)]),
            Self::Limit(limit) => (11, vec![text(operation), Value::Integer(limit)]),
            Self::Encoding(path, detail) => (10, vec![text(&path), text(&detail)]),
            Self::Io(path, error)
                if error.kind() == io::ErrorKind::InvalidInput
                    && error.raw_os_error().is_none()
                    && path.contains('\0') =>
            {
                (8, vec![text("Path"), text(&error.to_string())])
            }
            Self::Io(path, error) => {
                let variant = match error.kind() {
                    io::ErrorKind::NotFound => 0,
                    io::ErrorKind::PermissionDenied => 1,
                    io::ErrorKind::AlreadyExists => 2,
                    io::ErrorKind::NotADirectory => 3,
                    io::ErrorKind::IsADirectory => 4,
                    io::ErrorKind::DirectoryNotEmpty => 5,
                    io::ErrorKind::ReadOnlyFilesystem => 6,
                    _ => 7,
                };
                let cause = native_errors::host(
                    factory,
                    operation,
                    std::slice::from_ref(&path),
                    error.raw_os_error(),
                    error.to_string(),
                    location,
                )?;
                (
                    variant,
                    if variant == 7 {
                        vec![cause]
                    } else {
                        vec![text(&path), cause]
                    },
                )
            }
        };
        native_errors::value(factory, NativeErrorFamily::Fs, variant, fields, location)
    }
}
