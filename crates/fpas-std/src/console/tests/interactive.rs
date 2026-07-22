use super::*;

struct FailingWriter {
    successful_writes: usize,
    fail_after: usize,
}

impl std::io::Write for FailingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.successful_writes >= self.fail_after {
            return Err(std::io::Error::other("injected terminal write failure"));
        }
        self.successful_writes += 1;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
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
    }));
    let mut key_input = KeyInput::new();

    let error = console
        .acquire_interactive_terminal(&mut key_input, test_location())
        .unwrap_err();

    assert!(error.message.contains("terminal"));
    assert!(!console.interactive_terminal_acquired());
    assert_eq!(console.interactive, InteractiveTerminalOwnership::default());
}
