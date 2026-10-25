//! Explicit terminal streams for interactive child processes in CLI and embedding hosts.

use std::io::{self, IsTerminal};
use std::process::Stdio;
use std::sync::Arc;

/// Owned standard streams for one interactive child launch.
pub struct InteractiveProcessStdio {
    pub(crate) stdin: Stdio,
    pub(crate) stdout: Stdio,
    pub(crate) stderr: Stdio,
    #[cfg(unix)]
    pub(crate) foreground: Option<std::fs::File>,
}

impl InteractiveProcessStdio {
    /// Supply a host-owned terminal or independent scripted streams.
    #[must_use]
    pub fn new(stdin: Stdio, stdout: Stdio, stderr: Stdio) -> Self {
        Self {
            stdin,
            stdout,
            stderr,
            #[cfg(unix)]
            foreground: None,
        }
    }

    /// Transfer Unix foreground ownership during the child lifetime and restore it afterward.
    /// The file must refer to the calling host's foreground controlling terminal.
    #[cfg(unix)]
    #[must_use]
    pub fn with_foreground_terminal(mut self, terminal: std::fs::File) -> Self {
        self.foreground = Some(terminal);
        self
    }
}

/// A trusted host provider opening separate terminal streams for each launch.
///
/// No provider is installed by default. Debugger protocol input and queued
/// `Std.Console` input are never used as child-process input.
#[derive(Clone)]
pub struct InteractiveProcessTerminal {
    open: Arc<dyn Fn() -> io::Result<InteractiveProcessStdio> + Send + Sync>,
}

impl InteractiveProcessTerminal {
    /// Create a provider for an embedding host's terminal implementation.
    #[must_use]
    pub fn new(
        open: impl Fn() -> io::Result<InteractiveProcessStdio> + Send + Sync + 'static,
    ) -> Self {
        Self {
            open: Arc::new(open),
        }
    }

    /// Offer inherited CLI streams only when all three are actual terminals.
    #[must_use]
    pub fn inherited() -> Option<Self> {
        if !(io::stdin().is_terminal() && io::stdout().is_terminal() && io::stderr().is_terminal())
        {
            return None;
        }
        Some(Self::new(|| {
            let streams =
                InteractiveProcessStdio::new(Stdio::inherit(), Stdio::inherit(), Stdio::inherit());
            #[cfg(unix)]
            {
                use std::os::fd::AsFd;
                let terminal = io::stdin().as_fd().try_clone_to_owned()?;
                Ok(streams.with_foreground_terminal(terminal.into()))
            }
            #[cfg(not(unix))]
            Ok(streams)
        }))
    }

    pub(crate) fn open(&self) -> io::Result<InteractiveProcessStdio> {
        (self.open)()
    }
}
