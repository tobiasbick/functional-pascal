use super::*;
use std::sync::{Arc, Mutex};

struct FailingWriter {
    successful_writes: usize,
    fail_after: usize,
    failed: bool,
}

impl std::io::Write for FailingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.successful_writes >= self.fail_after && !self.failed {
            self.failed = true;
            return Err(std::io::Error::other("injected terminal write failure"));
        }
        self.successful_writes += 1;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct FailOnceWriterState {
    failures_remaining: usize,
    bytes: Vec<u8>,
}

struct FailOnceWriter {
    state: Arc<Mutex<FailOnceWriterState>>,
}

impl std::io::Write for FailOnceWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut state = self.state.lock().unwrap();
        if state.failures_remaining > 0 {
            state.failures_remaining -= 1;
            return Err(std::io::Error::other("injected one-shot terminal failure"));
        }
        state.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn console_with_fail_once_writer() -> (Console, Arc<Mutex<FailOnceWriterState>>) {
    let state = Arc::new(Mutex::new(FailOnceWriterState {
        failures_remaining: 1,
        bytes: Vec::new(),
    }));
    let writer = FailOnceWriter {
        state: Arc::clone(&state),
    };
    (Console::with_writer(Box::new(writer)), state)
}

#[test]
fn acquire_interactive_terminal_without_writer_is_exclusive_and_idempotent_on_release() {
    let mut console = Console::new();
    let mut key_input = KeyInput::new();

    console
        .acquire_interactive_terminal(&mut key_input, test_location())
        .unwrap();
    assert!(console.interactive_terminal_acquired());

    let second = console
        .acquire_interactive_terminal(&mut key_input, test_location())
        .unwrap_err();
    assert!(
        second
            .message
            .contains("second interactive terminal session")
    );

    console
        .release_interactive_terminal(&mut key_input, test_location())
        .unwrap();
    assert!(!console.interactive_terminal_acquired());

    console
        .release_interactive_terminal(&mut key_input, test_location())
        .unwrap();
}

#[test]
fn acquire_interactive_terminal_with_writer_tracks_owned_modes() {
    let (mut console, bytes) = console_with_shared_writer();
    let mut key_input = KeyInput::new();

    console
        .acquire_interactive_terminal(&mut key_input, test_location())
        .unwrap();
    assert!(console.interactive_terminal_acquired());
    assert!(!bytes.lock().unwrap().is_empty());

    console
        .release_interactive_terminal(&mut key_input, test_location())
        .unwrap();
    assert!(!console.interactive_terminal_acquired());
}

#[test]
fn acquire_interactive_terminal_rolls_back_owned_state_after_output_failure() {
    let mut console = Console::with_writer(Box::new(FailingWriter {
        successful_writes: 0,
        fail_after: 1,
        failed: false,
    }));
    let mut key_input = KeyInput::new();

    let error = console
        .acquire_interactive_terminal(&mut key_input, test_location())
        .unwrap_err();

    assert!(error.message.contains("terminal"));
    assert!(!console.interactive_terminal_acquired());
    assert_eq!(console.interactive, InteractiveTerminalOwnership::default());
}

#[test]
fn release_failure_retains_only_failed_mode_for_retry() {
    let (mut console, state) = console_with_fail_once_writer();
    let mut key_input = KeyInput::new();
    console.interactive = InteractiveTerminalOwnership {
        acquired: true,
        owns_paste: true,
        owns_mouse: true,
        ..InteractiveTerminalOwnership::default()
    };

    let error = console
        .release_interactive_terminal(&mut key_input, test_location())
        .expect_err("first paste restoration must fail");

    assert!(error.message.contains("injected one-shot terminal failure"));
    assert!(console.interactive.acquired);
    assert!(console.interactive.owns_paste);
    assert!(!console.interactive.owns_mouse);

    console
        .release_interactive_terminal(&mut key_input, test_location())
        .expect("retry must restore the retained paste mode");

    assert_eq!(console.interactive, InteractiveTerminalOwnership::default());
    assert!(!state.lock().unwrap().bytes.is_empty());
}

#[test]
fn console_drop_retries_a_transient_release_failure() {
    let (mut console, state) = console_with_fail_once_writer();
    let mut key_input = KeyInput::new();
    console.interactive = InteractiveTerminalOwnership {
        acquired: true,
        owns_alt_screen: true,
        ..InteractiveTerminalOwnership::default()
    };

    console
        .release_interactive_terminal(&mut key_input, test_location())
        .expect_err("first alternate-screen restoration must fail");
    assert!(console.interactive.owns_alt_screen);

    drop(console);

    assert!(!state.lock().unwrap().bytes.is_empty());
}
