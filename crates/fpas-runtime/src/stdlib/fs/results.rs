//! Convert structured filesystem outcomes at the native call boundary.
use super::failure::Failure;
use crate::stdlib::native_errors::{self, NativeErrorFamily};
use crate::stdlib::{AggregateFactory, StdError};
use crate::{SourceLocation, Value};

pub(super) fn result<T>(
    factory: &dyn AggregateFactory,
    operation: &str,
    resources: &[String],
    outcome: Result<T, Failure>,
    convert: impl FnOnce(T) -> Value,
    location: SourceLocation,
) -> Result<Value, StdError> {
    let error = match outcome {
        Ok(value) => return Ok(Value::result_ok(convert(value))),
        Err(error) => error,
    };
    let text = |s: &str| Value::Str(s.into());
    let primary = resources.first().map_or("", String::as_str);
    let (error, primary) = match error {
        Failure::AtPath(path, error) => (Failure::Io(error), path),
        Failure::EncodingAtPath(path, detail) => (Failure::InvalidEncoding(detail), path),
        other => (other, primary.to_owned()),
    };
    let (variant, fields) = match error {
        Failure::Io(error)
            if error.kind() == std::io::ErrorKind::InvalidInput
                && error.raw_os_error().is_none()
                && resources.iter().any(|path| path.contains('\0')) =>
        {
            let name = if resources.len() > 1 {
                if resources[0].contains('\0') {
                    "Source"
                } else {
                    "Target"
                }
            } else {
                "Path"
            };
            (8, vec![text(name), text(&error.to_string())])
        }
        Failure::Io(error) => {
            use std::io::ErrorKind::*;
            let variant = match error.kind() {
                NotFound => 0,
                PermissionDenied => 1,
                AlreadyExists => 2,
                NotADirectory => 3,
                IsADirectory => 4,
                DirectoryNotEmpty => 5,
                ReadOnlyFilesystem => 6,
                _ => 7,
            };
            let cause = native_errors::host(
                factory,
                operation,
                resources,
                error.raw_os_error(),
                error.to_string(),
                location,
            )?;
            (
                variant,
                if variant == 7 {
                    vec![cause]
                } else {
                    vec![text(&primary), cause]
                },
            )
        }
        Failure::AtPath(_, _) | Failure::EncodingAtPath(_, _) => {
            unreachable!("path errors are normalized above")
        }
        Failure::InvalidArgument(name, reason) => (8, vec![text(name), text(&reason)]),
        Failure::InvalidPattern(reason) => (9, vec![text(&primary), text(&reason)]),
        Failure::InvalidEncoding(detail) => (10, vec![text(&primary), text(&detail)]),
        Failure::LimitExceeded(operation, limit) => {
            (11, vec![text(operation), Value::Integer(limit)])
        }
        Failure::InvalidByte { offset, value } => (
            12,
            vec![Value::Integer(offset as i64), Value::Integer(value)],
        ),
    };
    native_errors::value(factory, NativeErrorFamily::Fs, variant, fields, location)
        .map(Value::result_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn native_categories_and_original_resources_survive_the_boundary() {
        for (kind, expected) in [
            (ErrorKind::NotFound, "FileNotFound"),
            (ErrorKind::PermissionDenied, "AccessDenied"),
            (ErrorKind::AlreadyExists, "AlreadyExists"),
            (ErrorKind::NotADirectory, "NotDirectory"),
            (ErrorKind::IsADirectory, "IsDirectory"),
            (ErrorKind::DirectoryNotEmpty, "NotEmpty"),
            (ErrorKind::ReadOnlyFilesystem, "ReadOnly"),
            (ErrorKind::Other, "IoFailure"),
        ] {
            let resources = ["source".into(), "target".into()];
            let returned = result(
                &crate::stdlib::intrinsics::TEST_AGGREGATES,
                "Std.Fs.CopyFile",
                &resources,
                Err::<(), _>(Failure::Io(Error::new(kind, "unrelated explanation"))),
                |()| Value::Boolean(true),
                SourceLocation::new(1, 1),
            )
            .expect("declared error layout");
            let Value::ResultError(error) = returned else {
                panic!("error result");
            };
            let Value::Enum(error) = error.as_ref() else {
                panic!("typed error");
            };
            assert_eq!(error.body().layout.variant, expected);
            let Value::Enum(cause) = error.body().values.last().expect("cause") else {
                panic!("host cause");
            };
            assert_eq!(cause.body().layout.type_name, "Std.Errors.HostError");
            assert_eq!(cause.body().values[0], Value::Str("Std.Fs.CopyFile".into()));
            assert_eq!(
                cause.body().values[1],
                Value::Array(vec![Value::Str("source".into()), Value::Str("target".into())].into())
            );
            assert_eq!(cause.body().values[2], Value::OptionNone);
            assert_eq!(cause.body().values[3], Value::OptionNone);
            assert_eq!(
                cause.body().values[4],
                Value::Str("unrelated explanation".into())
            );
        }
    }
}
