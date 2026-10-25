//! Explicit ceilings shared by both physical transports and connection admission.

use super::Fault;

/// Per-connection resource limits. Byte budgets count retained representations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Pending outgoing requests.
    pub outgoing_requests: usize,
    /// Pending incoming responders.
    pub incoming_requests: usize,
    /// Retained sessions, including observed restricted sessions.
    pub sessions: usize,
    /// Retained terminals, including exited unreleased terminals.
    pub terminals: usize,
    /// Retained MCP routes across all scopes.
    pub routes: usize,
    /// Entries in each bounded event/admission queue.
    pub queue_entries: usize,
    /// Entries admitted for outbound publication, including SDK-owned work.
    pub outbound_entries: usize,
    /// Retained bytes in each bounded event/admission queue.
    pub queue_bytes: usize,
    /// Complete UTF-8 message including its JSON-RPC envelope.
    pub message_bytes: usize,
    /// Retained agent stderr; input is always drained.
    pub stderr_bytes: usize,
    /// Cooperative shutdown grace before process-group termination.
    pub shutdown_millis: u64,
    /// Per-terminal retained output budget.
    pub terminal_bytes: usize,
    /// Aggregate reserved terminal output budget.
    pub all_terminal_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            outgoing_requests: 128,
            incoming_requests: 128,
            sessions: 64,
            terminals: 16,
            routes: 128,
            queue_entries: 256,
            outbound_entries: 128,
            queue_bytes: 16 * 1024 * 1024,
            message_bytes: 16 * 1024 * 1024,
            stderr_bytes: 256 * 1024,
            shutdown_millis: 2000,
            terminal_bytes: 1024 * 1024,
            all_terminal_bytes: 16 * 1024 * 1024,
        }
    }
}

impl Limits {
    /// Validate the complete configuration before allocating a connection or child.
    pub fn validate(&self) -> Result<(), Fault> {
        for (name, value, minimum, maximum) in [
            ("OutgoingRequests", self.outgoing_requests, 1, 4096),
            ("IncomingRequests", self.incoming_requests, 1, 4096),
            ("Sessions", self.sessions, 1, 4096),
            ("Terminals", self.terminals, 1, 256),
            ("Routes", self.routes, 1, 4096),
            ("QueueEntries", self.queue_entries, 1, 4096),
            ("OutboundEntries", self.outbound_entries, 1, 4096),
            ("QueueBytes", self.queue_bytes, 1, 64 * 1024 * 1024),
            ("MessageBytes", self.message_bytes, 1024, 64 * 1024 * 1024),
            ("StderrBytes", self.stderr_bytes, 0, 4 * 1024 * 1024),
            ("TerminalBytes", self.terminal_bytes, 0, 16 * 1024 * 1024),
            (
                "AllTerminalBytes",
                self.all_terminal_bytes,
                0,
                64 * 1024 * 1024,
            ),
        ] {
            if !(minimum..=maximum).contains(&value) {
                return Err(Fault::InvalidArgument(name));
            }
        }
        if self.shutdown_millis > 60000 {
            return Err(Fault::InvalidArgument("ShutdownMillis"));
        }
        Ok(())
    }
}
