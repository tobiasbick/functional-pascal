use super::*;
use std::cell::Cell;
fn location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

#[test]
fn cancellation_does_not_consume_queued_input() {
    let mut input = KeyInput::without_os_events();
    input.push_chars("x");
    assert!(input.read_key_controlled(location(), || true).is_err());
    assert_eq!(input.read_key(location()).unwrap(), 'x');
    input.push_console_event(ConsoleEvent::resize(80, 24));
    assert!(
        input
            .read_event_timeout_controlled(100, location(), || true)
            .is_err()
    );
    assert_eq!(
        input.poll_event(location()).unwrap(),
        Some(ConsoleEvent::resize(80, 24))
    );
}

#[test]
fn long_terminal_wait_observes_cancellation_after_a_bounded_probe() {
    struct Source<'a>(&'a Cell<bool>);
    impl TerminalEventSource for Source<'_> {
        fn poll(&mut self, timeout: Duration) -> std::io::Result<bool> {
            assert!(timeout <= Duration::from_millis(10));
            self.0.set(true);
            Ok(false)
        }
        fn read(&mut self) -> std::io::Result<crossterm::event::Event> {
            panic!("an unready terminal must never be read")
        }
    }
    let stop = Cell::new(false);
    let mut input = KeyInput::new();
    input.raw_mode = true;
    let result = input.read_event_timeout_controlled_from(
        3_600_000,
        location(),
        || stop.get(),
        &mut Source(&stop),
    );
    input.raw_mode = false;
    assert_eq!(
        result.unwrap_err().code,
        fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED
    );
}
