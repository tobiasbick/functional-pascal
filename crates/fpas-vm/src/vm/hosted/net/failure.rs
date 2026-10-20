//! Structured network failures retain native categories until the public Result boundary.
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::native_errors::{self, NativeErrorFamily};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(in crate::vm) enum NativeKind {
    Timeout = 1,
    ResolveFailed = 4,
    ConnectionRefused = 5,
    ConnectionLost = 6,
    AddressInUse = 7,
    AccessDenied = 8,
    IoFailure = 10,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::vm) enum Failure {
    Closed,
    Cancelled,
    InvalidArgument {
        name: &'static str,
        reason: String,
    },
    Native {
        kind: NativeKind,
        code: Option<i32>,
        detail: String,
    },
    Tls(String),
}

impl Failure {
    pub(in crate::vm) fn argument(name: &'static str, reason: impl Into<String>) -> Self {
        Self::InvalidArgument {
            name,
            reason: reason.into(),
        }
    }
    pub(in crate::vm) fn io(error: &io::Error, detail: String) -> Self {
        if error
            .get_ref()
            .is_some_and(|inner| inner.downcast_ref::<rustls::Error>().is_some())
        {
            return Self::Tls(detail);
        }
        let kind = match error.kind() {
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => NativeKind::Timeout,
            io::ErrorKind::ConnectionRefused => NativeKind::ConnectionRefused,
            io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::NotConnected => NativeKind::ConnectionLost,
            io::ErrorKind::AddrInUse => NativeKind::AddressInUse,
            io::ErrorKind::PermissionDenied => NativeKind::AccessDenied,
            _ => NativeKind::IoFailure,
        };
        Self::Native {
            kind,
            code: error.raw_os_error(),
            detail,
        }
    }
    pub(in crate::vm) fn resolve(error: &io::Error, detail: String) -> Self {
        Self::Native {
            kind: NativeKind::ResolveFailed,
            code: error.raw_os_error(),
            detail,
        }
    }
    pub(in crate::vm) fn no_addresses(detail: String) -> Self {
        Self::Native {
            kind: NativeKind::ResolveFailed,
            code: None,
            detail,
        }
    }
    pub(in crate::vm) fn deadline(detail: impl Into<String>) -> Self {
        Self::Native {
            kind: NativeKind::Timeout,
            code: None,
            detail: detail.into(),
        }
    }
    pub(super) fn pem(error: &rustls::pki_types::pem::Error, detail: String) -> Self {
        match error {
            rustls::pki_types::pem::Error::Io(error) => Self::io(error, detail),
            _ => Self::Tls(detail),
        }
    }
    /// Construct the admitted nominal family; missing metadata remains a VM invariant failure.
    pub(in crate::vm) fn into_value(
        self,
        factory: &dyn fpas_std::AggregateFactory,
        operation: &str,
        resources: &[String],
        location: SourceLocation,
    ) -> Result<Value, fpas_std::StdError> {
        let (variant, fields) = match self {
            Self::Closed => (0, vec![Value::Str(operation.into())]),
            Self::Cancelled => (2, vec![Value::Str(operation.into())]),
            Self::InvalidArgument { name, reason } => {
                (3, vec![Value::Str(name.into()), Value::Str(reason.into())])
            }
            Self::Tls(detail) => (9, vec![Value::Str(detail.into())]),
            Self::Native { kind, code, detail } => (
                kind as usize,
                vec![native_errors::host(
                    factory, operation, resources, code, detail, location,
                )?],
            ),
        };
        native_errors::value(factory, NativeErrorFamily::Net, variant, fields, location)
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => f.write_str("Network resource is closed"),
            Self::Cancelled => f.write_str("Network operation cancelled"),
            Self::InvalidArgument { reason, .. } => f.write_str(reason),
            Self::Native { detail, .. } | Self::Tls(detail) => f.write_str(detail),
        }
    }
}
