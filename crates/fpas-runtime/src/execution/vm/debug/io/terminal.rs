//! Thread-safe terminal bridge shared by the debugger actor and protocol host.

use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::execution::vm::hosted::terminal_input::{
    DebugTerminalEvent, DebugTerminalKeyEvent, DebugTerminalKeyKind, queue_event,
};
use crate::stdlib::{Console, KeyInput};

use super::DebuggeeChannel;
use crate::execution::vm::debug::types::{DebugErrorKind, DebugSessionError, DebuggeeInputResult};

#[derive(Default)]
struct TerminalOutput {
    pending: Mutex<Vec<u8>>,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
    total_bytes: AtomicUsize,
}

struct TerminalWriter {
    output: Arc<TerminalOutput>,
}

impl Write for TerminalWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut writer = self.output.writer.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(writer) = writer.as_mut() {
            writer.write_all(bytes)?;
            // A following native child inherits the same pipe, so flush before it runs.
            writer.flush()?;
        } else {
            self.output
                .pending
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .extend_from_slice(bytes);
        }
        self.output
            .total_bytes
            .fetch_add(bytes.len(), Ordering::Relaxed);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Cloneable debugger-side access to terminal output and input queues.
#[derive(Clone)]
pub struct DebugTerminalHandle {
    output: Arc<TerminalOutput>,
    console: Arc<Mutex<Console>>,
    key_input: Arc<Mutex<KeyInput>>,
    debuggee: Arc<Mutex<DebuggeeChannel>>,
}

impl DebugTerminalHandle {
    pub(in crate::execution::vm::debug) fn create(
        debuggee: Arc<Mutex<DebuggeeChannel>>,
    ) -> (Self, Arc<Mutex<Console>>, Arc<Mutex<KeyInput>>) {
        let output = Arc::new(TerminalOutput::default());
        let console = Arc::new(Mutex::new(Console::with_capturing_writer(Box::new(
            TerminalWriter {
                output: Arc::clone(&output),
            },
        ))));
        let key_input = Arc::new(Mutex::new(KeyInput::without_os_events()));
        (
            Self {
                output,
                console: Arc::clone(&console),
                key_input: Arc::clone(&key_input),
                debuggee,
            },
            console,
            key_input,
        )
    }

    /// Route future terminal bytes directly to an owned host writer instead of buffering.
    /// Batch hosts use their isolated worker pipe so native and hosted stdout share order.
    pub fn set_output_writer(&self, writer: Box<dyn Write + Send>) {
        *self.output.writer.lock().unwrap_or_else(|p| p.into_inner()) = Some(writer);
    }

    /// Remove and return terminal bytes produced since the previous drain.
    #[must_use]
    pub fn take_output(&self) -> Vec<u8> {
        std::mem::take(
            &mut *self
                .output
                .pending
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    /// Return the total number of terminal bytes emitted by this session.
    #[must_use]
    pub fn output_byte_count(&self) -> usize {
        self.output.total_bytes.load(Ordering::Relaxed)
    }

    /// Queue terminal events without requiring ownership of the running debug session.
    ///
    /// # Errors
    ///
    /// Returns a debugger input error after EOF, disconnect, or the configured input limit.
    pub fn push_events(
        &self,
        events: &[DebugTerminalEvent],
    ) -> Result<DebuggeeInputResult, DebugSessionError> {
        let bytes = events.iter().map(event_size).sum::<usize>();
        let session_bytes = self
            .debuggee
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .accept_input(bytes)
            .map_err(terminal_input_error)?;
        let mut console = self
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut input = self
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for event in events {
            queue_event(&mut console, &mut input, event);
        }
        Ok(DebuggeeInputResult {
            bytes,
            session_bytes,
        })
    }
}

fn event_size(event: &DebugTerminalEvent) -> usize {
    match event {
        DebugTerminalEvent::Paste(text) => text.len(),
        DebugTerminalEvent::Key(DebugTerminalKeyEvent {
            kind: DebugTerminalKeyKind::Character(character),
            ..
        }) => character.len_utf8(),
        _ => 1,
    }
}

fn terminal_input_error(kind: DebugErrorKind) -> DebugSessionError {
    match kind {
        DebugErrorKind::DebuggeeInputLimit => DebugSessionError {
            kind,
            message: "debuggee terminal input exceeds the session input limit".to_string(),
            hint: "Send fewer terminal events, or raise the debugger input limit.".to_string(),
        },
        DebugErrorKind::DebuggeeInputClosed => DebugSessionError {
            kind,
            message: "debuggee terminal input is closed".to_string(),
            hint: "Do not send terminal events after EOF or disconnect.".to_string(),
        },
        _ => DebugSessionError {
            kind: DebugErrorKind::InvalidState,
            message: "debuggee terminal input is unavailable".to_string(),
            hint: "Send terminal input only while the debug session is connected.".to_string(),
        },
    }
}
