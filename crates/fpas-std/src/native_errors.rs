//! Shared nominal error catalog for hosted operations and parsers.

mod proc;

/// Public native error family, indexed once in an admitted executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum NativeErrorFamily {
    /// `Std.Errors.HostError`.
    Host,
    /// `Std.Fs.FsError`.
    Fs,
    /// `Std.Proc.ProcError`.
    Proc,
    /// `Std.Parse.ParseError`.
    Parse,
    /// `Std.Json.JsonError`.
    Json,
    /// `Std.Toml.TomlError`.
    Toml,
    /// `Std.Crypto.CryptoError`.
    Crypto,
    /// `Std.Tasks.TaskError`.
    Task,
    /// `Std.Net.NetError`.
    Net,
    /// `Std.Server.ServerError`.
    Server,
    /// `Std.Encoding.EncodingError`.
    Encoding,
}

/// Payload type in a native error declaration.
#[derive(Debug, Clone, Copy)]
pub enum ErrorFieldType {
    /// Text.
    String,
    /// Text resources.
    Strings,
    /// Optional text.
    OptionalString,
    /// Optional native code or parser position.
    OptionalInteger,
    /// Integer limit.
    Integer,
    /// Shared native cause.
    Host,
    /// Network operation cause.
    Net,
}
impl ErrorFieldType {
    /// Native family carried by a typed Cause payload.
    pub const fn cause_family(self) -> Option<NativeErrorFamily> {
        match self {
            Self::Host => Some(NativeErrorFamily::Host),
            Self::Net => Some(NativeErrorFamily::Net),
            _ => None,
        }
    }
}
/// Static variant metadata shared with semantic registration.
pub struct ErrorVariant {
    /// Declaration name.
    pub name: &'static str,
    /// Automatic display label.
    pub message: &'static str,
    /// Ordered payload names and types.
    pub fields: &'static [(&'static str, ErrorFieldType)],
}
impl NativeErrorFamily {
    /// Every native family in stable catalog order.
    pub const ALL: &'static [Self] = &[
        Self::Host,
        Self::Fs,
        Self::Proc,
        Self::Parse,
        Self::Json,
        Self::Toml,
        Self::Crypto,
        Self::Task,
        Self::Net,
        Self::Server,
        Self::Encoding,
    ];
    /// Qualified nominal name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Host => "Std.Errors.HostError",
            Self::Fs => "Std.Fs.FsError",
            Self::Proc => "Std.Proc.ProcError",
            Self::Parse => "Std.Parse.ParseError",
            Self::Json => "Std.Json.JsonError",
            Self::Toml => "Std.Toml.TomlError",
            Self::Crypto => "Std.Crypto.CryptoError",
            Self::Task => crate::std_symbols::STD_TASK_TASK_ERROR,
            Self::Net => crate::std_symbols::STD_NET_ERROR,
            Self::Server => crate::std_symbols::STD_SERVER_ERROR,
            Self::Encoding => crate::std_symbols::STD_ENCODING_ERROR,
        }
    }
    /// Ordered declared variants.
    pub const fn variants(self) -> &'static [ErrorVariant] {
        use ErrorFieldType::*;
        match self {
            Self::Encoding => &[
                ErrorVariant {
                    name: "InvalidByte",
                    message: "Byte value is outside 0..255",
                    fields: &[("Offset", Integer), ("Value", Integer)],
                },
                ErrorVariant {
                    name: "InvalidCharacter",
                    message: "Invalid encoding character",
                    fields: &[("Offset", Integer)],
                },
                ErrorVariant {
                    name: "InvalidLength",
                    message: "Invalid encoding length",
                    fields: &[("Length", Integer)],
                },
                ErrorVariant {
                    name: "InvalidPadding",
                    message: "Invalid Base64 padding",
                    fields: &[("Offset", Integer)],
                },
                ErrorVariant {
                    name: "NonCanonical",
                    message: "Noncanonical Base64 text",
                    fields: &[("Offset", Integer)],
                },
                ErrorVariant {
                    name: "LimitExceeded",
                    message: "Encoding size limit exceeded",
                    fields: &[("Limit", Integer)],
                },
            ],
            Self::Task => &[
                ErrorVariant {
                    name: "Closed",
                    message: "Channel is closed",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "Timeout",
                    message: "Operation timed out",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "Cancelled",
                    message: "Operation cancelled",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "InvalidArgument",
                    message: "Task operation failed",
                    fields: &[("Name", String), ("Reason", String)],
                },
                ErrorVariant {
                    name: "InvalidGroup",
                    message: "Task operation failed",
                    fields: &[("Operation", String)],
                },
            ],
            Self::Net => &[
                ErrorVariant {
                    name: "Closed",
                    message: "Connection is closed",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "Timeout",
                    message: "Operation timed out",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "Cancelled",
                    message: "Operation cancelled",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "InvalidArgument",
                    message: "Network operation failed",
                    fields: &[("Name", String), ("Reason", String)],
                },
                ErrorVariant {
                    name: "ResolveFailed",
                    message: "Network operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "ConnectionRefused",
                    message: "Network operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "ConnectionLost",
                    message: "Network operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "AddressInUse",
                    message: "Network operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "AccessDenied",
                    message: "Network operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "TlsFailure",
                    message: "Network operation failed",
                    fields: &[("Detail", String)],
                },
                ErrorVariant {
                    name: "IoFailure",
                    message: "Network operation failed",
                    fields: &[("Cause", Host)],
                },
            ],
            Self::Server => &[
                ErrorVariant {
                    name: "InvalidArgument",
                    message: "Server operation failed",
                    fields: &[("Name", String), ("Reason", String)],
                },
                ErrorVariant {
                    name: "InvalidState",
                    message: "Server operation failed",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "ListenerFailure",
                    message: "Server operation failed",
                    fields: &[("Cause", Net)],
                },
                ErrorVariant {
                    name: "SignalFailure",
                    message: "Server operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "Timeout",
                    message: "Operation timed out",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "IoFailure",
                    message: "Server operation failed",
                    fields: &[("Cause", Host)],
                },
            ],
            Self::Host => &[ErrorVariant {
                name: "Failure",
                message: "Host operation failed",
                fields: &[
                    ("Operation", String),
                    ("Resources", Strings),
                    ("Domain", OptionalString),
                    ("Code", OptionalInteger),
                    ("Detail", String),
                ],
            }],
            Self::Fs => &[
                ErrorVariant {
                    name: "FileNotFound",
                    message: "File not found",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "AccessDenied",
                    message: "Access denied",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "AlreadyExists",
                    message: "Filesystem operation failed",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "NotDirectory",
                    message: "Filesystem operation failed",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "IsDirectory",
                    message: "Filesystem operation failed",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "NotEmpty",
                    message: "Filesystem operation failed",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "ReadOnly",
                    message: "Filesystem operation failed",
                    fields: &[("Path", String), ("Cause", Host)],
                },
                ErrorVariant {
                    name: "IoFailure",
                    message: "Filesystem operation failed",
                    fields: &[("Cause", Host)],
                },
                ErrorVariant {
                    name: "InvalidArgument",
                    message: "Filesystem operation failed",
                    fields: &[("Name", String), ("Reason", String)],
                },
                ErrorVariant {
                    name: "InvalidPattern",
                    message: "Filesystem operation failed",
                    fields: &[("Pattern", String), ("Reason", String)],
                },
                ErrorVariant {
                    name: "InvalidEncoding",
                    message: "Filesystem operation failed",
                    fields: &[("Path", String), ("Detail", String)],
                },
                ErrorVariant {
                    name: "LimitExceeded",
                    message: "Filesystem operation failed",
                    fields: &[("Operation", String), ("Limit", Integer)],
                },
                ErrorVariant {
                    name: "InvalidByte",
                    message: "Byte value is outside 0..255",
                    fields: &[("Offset", Integer), ("Value", Integer)],
                },
                ErrorVariant {
                    name: "Closed",
                    message: "File is closed",
                    fields: &[("Operation", String)],
                },
                ErrorVariant {
                    name: "Cancelled",
                    message: "Operation cancelled",
                    fields: &[("Operation", String)],
                },
            ],
            Self::Proc => proc::VARIANTS,
            Self::Parse => &[
                ErrorVariant {
                    name: "InvalidSyntax",
                    message: "Invalid value syntax",
                    fields: &[("Target", String), ("Detail", String)],
                },
                ErrorVariant {
                    name: "OutOfRange",
                    message: "Value outside supported range",
                    fields: &[("Target", String), ("Detail", String)],
                },
            ],
            Self::Json => &[ErrorVariant {
                name: "InvalidSyntax",
                message: "Invalid JSON",
                fields: &[
                    ("Line", OptionalInteger),
                    ("Column", OptionalInteger),
                    ("Detail", String),
                ],
            }],
            Self::Toml => &[ErrorVariant {
                name: "InvalidSyntax",
                message: "Invalid TOML",
                fields: &[
                    ("Line", OptionalInteger),
                    ("Column", OptionalInteger),
                    ("Detail", String),
                ],
            }],
            Self::Crypto => &[
                ErrorVariant {
                    name: "InvalidArgument",
                    message: "Secure random operation failed",
                    fields: &[("Name", String), ("Reason", String)],
                },
                ErrorVariant {
                    name: "EntropyUnavailable",
                    message: "Secure random operation failed",
                    fields: &[("Cause", Host)],
                },
            ],
        }
    }
}

/// Construct a declared native variant through the executing image's layout cache.
///
/// # Errors
/// Returns an internal diagnostic when the admitted image lacks the layout.
pub fn value(
    factory: &dyn crate::AggregateFactory,
    family: NativeErrorFamily,
    variant: usize,
    fields: Vec<fpas_bytecode::Value>,
    location: fpas_bytecode::SourceLocation,
) -> Result<fpas_bytecode::Value, crate::StdError> {
    factory.native_error(family, variant, fields, location)
}

/// Construct the original native host cause, without interpreting its explanation.
///
/// # Errors
/// Returns an internal diagnostic when the admitted image lacks HostError.
pub fn host(
    factory: &dyn crate::AggregateFactory,
    operation: &str,
    resources: &[String],
    code: Option<i32>,
    detail: String,
    location: fpas_bytecode::SourceLocation,
) -> Result<fpas_bytecode::Value, crate::StdError> {
    use fpas_bytecode::Value;
    let domain = code.map(|_| Value::Str(if cfg!(windows) { "win32" } else { "errno" }.into()));
    value(
        factory,
        NativeErrorFamily::Host,
        0,
        vec![
            Value::Str(operation.into()),
            Value::Array(
                resources
                    .iter()
                    .map(|resource| Value::Str(resource.clone().into()))
                    .collect(),
            ),
            domain.map_or(Value::OptionNone, Value::option_some),
            code.map_or(Value::OptionNone, |code| {
                Value::option_some(Value::Integer(i64::from(code)))
            }),
            Value::Str(detail.into()),
        ],
        location,
    )
}

/// Build a parser failure, retaining only positions supplied by the parser.
///
/// # Errors
/// Returns an internal diagnostic when the image lacks its declared error family.
pub fn syntax(
    factory: &dyn crate::AggregateFactory,
    family: NativeErrorFamily,
    line: Option<usize>,
    column: Option<usize>,
    detail: String,
    location: fpas_bytecode::SourceLocation,
) -> Result<fpas_bytecode::Value, crate::StdError> {
    use fpas_bytecode::Value;
    let position = |n: Option<usize>| {
        n.and_then(|n| i64::try_from(n).ok())
            .map_or(Value::OptionNone, |n| Value::option_some(Value::Integer(n)))
    };
    value(
        factory,
        family,
        0,
        vec![position(line), position(column), Value::Str(detail.into())],
        location,
    )
}
