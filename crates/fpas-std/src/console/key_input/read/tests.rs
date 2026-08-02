//! Deterministic live-event source regressions for timeout and poll reads.

use std::collections::VecDeque;
use std::io;
use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::super::KeyInput;
use super::super::event_source::TerminalEventSource;
use fpas_bytecode::SourceLocation;

#[derive(Default)]
struct FakeEventSource {
    events: VecDeque<Event>,
    poll_timeouts: Vec<Duration>,
}

impl FakeEventSource {
    fn with_events(events: impl IntoIterator<Item = Event>) -> Self {
        Self {
            events: events.into_iter().collect(),
            poll_timeouts: Vec::new(),
        }
    }
}

impl TerminalEventSource for FakeEventSource {
    fn poll(&mut self, timeout: Duration) -> io::Result<bool> {
        self.poll_timeouts.push(timeout);
        Ok(!self.events.is_empty())
    }

    fn read(&mut self) -> io::Result<Event> {
        self.events.pop_front().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "fake event queue is empty")
        })
    }
}

fn location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn raw_input() -> KeyInput {
    let mut input = KeyInput::new();
    input.raw_mode = true;
    input
}

fn key(kind: KeyEventKind, ch: char) -> Event {
    Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char(ch),
        KeyModifiers::NONE,
        kind,
    ))
}

#[test]
fn timeout_skips_release_and_returns_following_press_with_remaining_deadline() {
    let mut input = raw_input();
    let mut source = FakeEventSource::with_events([
        key(KeyEventKind::Release, 'x'),
        key(KeyEventKind::Press, 'x'),
    ]);

    let event = input
        .read_event_timeout_from(50, location(), &mut source)
        .expect("timeout read")
        .expect("press event");

    assert_eq!(event.key.ch, 'x');
    assert_eq!(source.poll_timeouts.len(), 2);
    assert!(source.poll_timeouts[1] <= source.poll_timeouts[0]);
}

#[test]
fn zero_timeout_drains_ready_release_before_press_at_deadline_boundary() {
    let mut input = raw_input();
    let mut source = FakeEventSource::with_events([
        key(KeyEventKind::Release, 'z'),
        key(KeyEventKind::Press, 'z'),
    ]);

    let event = input
        .read_event_timeout_from(0, location(), &mut source)
        .expect("non-blocking timeout read")
        .expect("ready press event");

    assert_eq!(event.key.ch, 'z');
    assert!(source.poll_timeouts.iter().all(Duration::is_zero));
}

#[test]
fn timeout_returns_none_after_ignored_release_when_deadline_has_no_press() {
    let mut input = raw_input();
    let mut source = FakeEventSource::with_events([key(KeyEventKind::Release, 'x')]);

    let event = input
        .read_event_timeout_from(0, location(), &mut source)
        .expect("non-blocking timeout read");

    assert!(event.is_none());
}

#[test]
fn poll_skips_release_and_returns_immediately_ready_press() {
    let mut input = raw_input();
    let mut source = FakeEventSource::with_events([
        key(KeyEventKind::Release, 'p'),
        key(KeyEventKind::Press, 'p'),
    ]);

    let event = input
        .poll_event_from(location(), &mut source)
        .expect("poll read")
        .expect("ready press event");

    assert_eq!(event.key.ch, 'p');
    assert!(source.poll_timeouts.iter().all(Duration::is_zero));
}
