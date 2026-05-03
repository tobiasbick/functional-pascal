use super::*;
use crate::console_event::{
    ConsoleEvent, event_kind_index, mouse_action_index, mouse_button_index,
};
use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crossterm::event::{
    Event, KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use fpas_bytecode::{SourceLocation, Value};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

mod colors;
mod input;
mod key_events;
mod screen;
mod terminal;

fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

#[derive(Clone)]
struct SharedBufferWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl Write for SharedBufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn console_with_shared_writer() -> (Console, Arc<Mutex<Vec<u8>>>) {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedBufferWriter {
        bytes: Arc::clone(&bytes),
    };
    (Console::with_writer(Box::new(writer)), bytes)
}
