//! Portable resource cleanup family identifiers.

/// Identifies the native release semantics encoded in `RegisterCleanup`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum CleanupFamily {
    /// A user-defined release procedure.
    Custom = 0,
    /// A file handle.
    File = 1,
    /// A network socket.
    Socket = 2,
    /// A network listener.
    Listener = 3,
    /// A process handle.
    Process = 4,
    /// An ACP connection.
    Acp = 5,
    /// An MCP server.
    Mcp = 6,
    /// An MCP endpoint.
    McpEndpoint = 7,
    /// An HTTP response stream.
    HttpStream = 8,
}

impl CleanupFamily {
    /// Decode a bytecode operand, rejecting unknown identifiers.
    pub fn from_u16(value: u16) -> Option<Self> {
        const FAMILIES: &[CleanupFamily] = &[
            CleanupFamily::Custom,
            CleanupFamily::File,
            CleanupFamily::Socket,
            CleanupFamily::Listener,
            CleanupFamily::Process,
            CleanupFamily::Acp,
            CleanupFamily::Mcp,
            CleanupFamily::McpEndpoint,
            CleanupFamily::HttpStream,
        ];
        FAMILIES
            .get(usize::from(value))
            .copied()
            .filter(|family| family.to_u16() == value)
    }

    /// Encode this family as a bytecode operand.
    pub const fn to_u16(self) -> u16 {
        self as u16
    }
}
