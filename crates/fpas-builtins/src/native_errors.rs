//! Shared nominal error catalog for hosted operations and parsers.

mod acp;
mod mcp;
mod proc;

/// Public native error family, indexed once in an admitted executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum NativeErrorFamily {
    /// `Std.Os.Errors.HostError`.
    Host,
    /// `Std.Io.Fs.FsError`.
    Fs,
    /// `Std.Os.Proc.ProcError`.
    Proc,
    /// `Std.Text.Parse.ParseError`.
    Parse,
    /// `Std.Encoding.Json.JsonError`.
    Json,
    /// `Std.Encoding.Toml.TomlError`.
    Toml,
    /// `Std.Crypto.CryptoError`.
    Crypto,
    /// `Std.Tasks.TaskError`.
    Task,
    /// `Std.Net.Sockets.NetError`.
    Net,
    /// `Std.Net.Server.ServerError`.
    Server,
    /// `Std.Encoding.EncodingError`.
    Encoding,
    /// `Std.Ai.Acp.Types.AcpError`.
    Acp,
    /// Independent MCP registration and serving failures.
    Mcp,
    /// `Std.Ai.Tokenizer.TokenizerError`.
    Tokenizer,
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
    /// Registered detached record.
    Record(&'static str),
    /// Optional registered detached record.
    OptionalRecord(&'static str),
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
    /// Whether ordinary display must omit associated values and nested causes.
    /// Explicit field inspection retains the complete returned error.
    pub const fn hide_payload(self) -> bool {
        matches!(self, Self::Acp | Self::Mcp)
    }

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
        Self::Acp,
        Self::Mcp,
        Self::Tokenizer,
    ];
    /// Qualified nominal name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Host => "Std.Os.Errors.HostError",
            Self::Fs => "Std.Io.Fs.FsError",
            Self::Proc => "Std.Os.Proc.ProcError",
            Self::Parse => "Std.Text.Parse.ParseError",
            Self::Json => "Std.Encoding.Json.JsonError",
            Self::Toml => "Std.Encoding.Toml.TomlError",
            Self::Crypto => "Std.Crypto.CryptoError",
            Self::Task => crate::std_symbols::STD_TASK_TASK_ERROR,
            Self::Net => crate::std_symbols::STD_NET_ERROR,
            Self::Server => crate::std_symbols::STD_SERVER_ERROR,
            Self::Encoding => crate::std_symbols::STD_ENCODING_ERROR,
            Self::Acp => "Std.Ai.Acp.Types.AcpError",
            Self::Mcp => "Std.Ai.Mcp.Types.McpError",
            Self::Tokenizer => "Std.Ai.Tokenizer.TokenizerError",
        }
    }
    /// Ordered declared variants.
    pub const fn variants(self) -> &'static [ErrorVariant] {
        use ErrorFieldType::*;
        match self {
            Self::Acp => acp::VARIANTS,
            Self::Mcp => mcp::VARIANTS,
            Self::Tokenizer => &[
                ErrorVariant {
                    name: "UnsupportedEncoding",
                    message: "Unsupported tokenizer encoding",
                    fields: &[("Name", String)],
                },
                ErrorVariant {
                    name: "InvalidToken",
                    message: "Invalid model token",
                    fields: &[("Index", Integer), ("Value", Integer)],
                },
                ErrorVariant {
                    name: "InvalidUtf8",
                    message: "Tokens do not form valid UTF-8",
                    fields: &[],
                },
                ErrorVariant {
                    name: "LimitExceeded",
                    message: "Tokenizer limit exceeded",
                    fields: &[("Name", String), ("Limit", Integer)],
                },
                ErrorVariant {
                    name: "BackendFailure",
                    message: "Tokenizer backend failed",
                    fields: &[],
                },
            ],
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
