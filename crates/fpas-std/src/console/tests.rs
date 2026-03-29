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

#[test]
fn text_input_read_then_readln_shares_buffer() {
    let mut t = TextInput::new();
    t.push_line("ab");
    assert_eq!(t.read_char(test_location()).unwrap(), 'a');
    assert_eq!(t.read_char(test_location()).unwrap(), 'b');
    // Rest of line is newline consumed by ReadLn -> empty string
    assert_eq!(t.read_line(test_location()).unwrap(), "");
}

#[test]
fn text_input_readln_then_read() {
    let mut t = TextInput::new();
    t.push_line("xy");
    assert_eq!(t.read_line(test_location()).unwrap(), "xy");
    t.push_line("z");
    assert_eq!(t.read_char(test_location()).unwrap(), 'z');
    assert_eq!(t.read_line(test_location()).unwrap(), "");
}

#[test]
fn key_input_test_queue_readkey_keypressed() {
    let mut k = KeyInput::new();
    assert!(!k.key_pressed(test_location()).unwrap());
    k.push_chars("a");
    assert!(k.key_pressed(test_location()).unwrap());
    assert_eq!(k.read_key(test_location()).unwrap(), 'a');
    assert!(!k.key_pressed(test_location()).unwrap());
}

#[test]
fn key_input_extended_sequence() {
    let mut k = KeyInput::new();
    k.push_chars("\0H"); // #0 then scan for Up (72 = 'H' in TP layout)
    assert_eq!(k.read_key(test_location()).unwrap(), '\0');
    assert_eq!(k.read_key(test_location()).unwrap(), 'H');
}

#[test]
fn key_input_key_pressed_sees_event_queue_without_char_queue() {
    let mut k = KeyInput::new();
    assert!(!k.key_pressed(test_location()).unwrap());
    k.push_key_event(ConsoleKeyEvent::new(0, '\0', false, false, false, false));
    assert!(k.key_pressed(test_location()).unwrap());
}

#[test]
fn key_input_read_key_event_fifo() {
    let mut k = KeyInput::new();
    k.push_key_event(ConsoleKeyEvent::new(7, '\0', true, false, false, false));
    k.push_key_event(ConsoleKeyEvent::new(8, '\0', false, true, false, false));
    let a = k.read_key_event(test_location()).unwrap();
    assert_eq!(a.kind, 7);
    assert!(a.shift);
    let b = k.read_key_event(test_location()).unwrap();
    assert_eq!(b.kind, 8);
    assert!(b.ctrl);
}

#[test]
fn key_input_read_key_does_not_consume_event_queue() {
    let mut k = KeyInput::new();
    k.push_key_event(ConsoleKeyEvent::new(5, ' ', false, false, false, false));
    k.push_chars("z");
    assert_eq!(k.read_key(test_location()).unwrap(), 'z');
    let ev = k.read_key_event(test_location()).unwrap();
    assert_eq!(ev.kind, 5);
    assert_eq!(ev.ch, ' ');
}

#[test]
fn key_input_live_queue_feeds_read_key_event() {
    let mut k = KeyInput::new();
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::SHIFT,
    ))));
    assert!(k.key_pressed(test_location()).unwrap());
    let ev = k.read_key_event(test_location()).unwrap();
    assert_eq!(ev.kind, key_kind_index("Character"));
    assert_eq!(ev.ch, 'x');
    assert!(ev.shift);
    assert!(!k.key_pressed(test_location()).unwrap());
}

#[test]
fn key_input_key_pressed_ignores_unified_only_events() {
    let mut k = KeyInput::new();
    k.push_console_event(ConsoleEvent::focus_gained());
    assert!(!k.key_pressed(test_location()).unwrap());
    assert!(k.event_pending(test_location()).unwrap());
}

#[test]
fn key_input_read_event_returns_queued_resize() {
    let mut k = KeyInput::new();
    k.push_console_event(ConsoleEvent::resize(120, 40));
    assert!(k.event_pending(test_location()).unwrap());
    let event = k.read_event(test_location()).unwrap();
    assert_eq!(event.kind, event_kind_index("Resize"));
    assert_eq!(event.width, 120);
    assert_eq!(event.height, 40);
}

#[test]
fn key_input_read_event_preserves_fifo_across_event_kinds() {
    let mut k = KeyInput::new();
    k.push_console_event(ConsoleEvent::paste("hello".into()));
    k.push_console_event(ConsoleEvent::focus_lost());

    let paste = k.read_event(test_location()).unwrap();
    assert_eq!(paste.kind, event_kind_index("Paste"));
    assert_eq!(paste.text, "hello");

    let focus = k.read_event(test_location()).unwrap();
    assert_eq!(focus.kind, event_kind_index("FocusLost"));
}

#[test]
fn key_input_live_mouse_event_maps_to_one_based_console_coordinates() {
    let mut k = KeyInput::new();
    assert!(k.push_live_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Right),
        column: 4,
        row: 2,
        modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL,
    })));

    assert!(k.event_pending(test_location()).unwrap());
    let event = k.read_event(test_location()).unwrap();
    assert_eq!(event.kind, event_kind_index("Mouse"));
    assert_eq!(event.mouse_action, mouse_action_index("Drag"));
    assert_eq!(event.mouse_button, mouse_button_index("Right"));
    assert_eq!(event.mouse_x, 5);
    assert_eq!(event.mouse_y, 3);
    assert!(event.shift);
    assert!(event.ctrl);
}

#[test]
fn key_input_live_key_event_is_visible_to_unified_event_api() {
    let mut k = KeyInput::new();
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('Z'),
        KeyModifiers::ALT | KeyModifiers::SHIFT,
    ))));

    assert!(k.event_pending(test_location()).unwrap());
    let event = k.read_event(test_location()).unwrap();
    assert_eq!(event.kind, event_kind_index("Key"));
    assert_eq!(event.key.kind, key_kind_index("Character"));
    assert_eq!(event.key.ch, 'Z');
    assert!(event.alt);
    assert!(event.shift);
}

#[test]
fn key_input_event_pending_is_false_when_all_queues_are_empty() {
    let mut k = KeyInput::new();
    assert!(!k.event_pending(test_location()).unwrap());
}

#[test]
fn console_window_coordinates_are_relative() {
    let mut c = Console::new();
    c.window(10, 5, 12, 6, test_location()).unwrap();
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
    c.goto_xy(2, 2, test_location()).unwrap();
    assert_eq!(c.where_x(), 2);
    assert_eq!(c.where_y(), 2);
    c.write(&Value::Char('X'), test_location()).unwrap();
    assert_eq!(c.test_line_text(6).chars().nth(10), Some('X'));
}

#[test]
fn console_clrscr_only_clears_active_window() {
    let mut c = Console::new();
    c.write(&Value::Str("ABCDE".into()), test_location())
        .unwrap();
    c.window(2, 1, 4, 1, test_location()).unwrap();
    c.clr_scr(test_location()).unwrap();
    let row = c.test_line_text(1);
    assert_eq!(row.chars().take(5).collect::<String>(), "A   E");
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
}

#[test]
fn console_scrolls_inside_active_window() {
    let mut c = Console::new();
    c.window(1, 1, 3, 2, test_location()).unwrap();
    c.write(&Value::Str("ab".into()), test_location()).unwrap();
    c.write(&Value::Str("c".into()), test_location()).unwrap();
    c.write(&Value::Str("de".into()), test_location()).unwrap();
    // "fg": 'f' fills (3,2) with a pending wrap; 'g' triggers the wrap → scroll.
    c.write(&Value::Str("fg".into()), test_location()).unwrap();
    assert_eq!(
        c.test_line_text(1).chars().take(3).collect::<String>(),
        "def"
    );
    assert_eq!(
        c.test_line_text(2).chars().take(3).collect::<String>(),
        "g  "
    );
    assert_eq!(c.where_x(), 2);
    assert_eq!(c.where_y(), 2);
}

#[test]
fn console_text_attributes_and_clreol_use_current_colors() {
    let mut c = Console::new();
    c.text_color(12, test_location()).unwrap();
    c.text_background(1, test_location()).unwrap();
    c.write(&Value::Str("xy".into()), test_location()).unwrap();
    c.clr_eol(test_location()).unwrap();
    let first = c.test_cell(1, 1);
    let cleared = c.test_cell(10, 1);
    assert_eq!(first.0, 'x');
    assert_eq!(first.1, 12);
    assert_eq!(first.2, 1);
    assert_eq!(cleared.0, ' ');
    assert_eq!(cleared.1, 12);
    assert_eq!(cleared.2, 1);
}

#[test]
fn console_del_line_and_ins_line_inside_window() {
    let mut c = Console::new();
    c.window(1, 1, 5, 3, test_location()).unwrap();
    c.write(&Value::Str("AAAAA".into()), test_location())
        .unwrap();
    c.goto_xy(1, 2, test_location()).unwrap();
    c.write(&Value::Str("BBBBB".into()), test_location())
        .unwrap();
    c.goto_xy(1, 3, test_location()).unwrap();
    c.write(&Value::Str("CCCCC".into()), test_location())
        .unwrap();

    c.goto_xy(1, 2, test_location()).unwrap();
    c.del_line(test_location()).unwrap();
    assert_eq!(
        c.test_line_text(2).chars().take(5).collect::<String>(),
        "CCCCC"
    );
    assert_eq!(
        c.test_line_text(3).chars().take(5).collect::<String>(),
        "     "
    );

    c.goto_xy(1, 2, test_location()).unwrap();
    c.ins_line(test_location()).unwrap();
    assert_eq!(
        c.test_line_text(2).chars().take(5).collect::<String>(),
        "     "
    );
    assert_eq!(
        c.test_line_text(3).chars().take(5).collect::<String>(),
        "CCCCC"
    );
}

#[test]
fn console_text_attr_and_video_helpers() {
    let mut c = Console::new();
    c.set_text_attr(0x1E, test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x1E);

    c.low_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x16);

    c.high_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x1E);

    c.norm_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x07);
}

#[test]
fn console_wind_min_and_wind_max_follow_window_and_resize() {
    let mut c = Console::new();
    assert_eq!(c.wind_min(), 0x0101);
    assert_eq!(c.wind_max(), 0x1950);

    c.window(10, 5, 20, 8, test_location()).unwrap();
    assert_eq!(c.wind_min(), 0x050A);
    assert_eq!(c.wind_max(), 0x0814);

    c.resize(12, 6);
    assert_eq!(c.wind_min(), 0x050A);
    assert_eq!(c.wind_max(), 0x060C);
}

#[test]
fn console_del_line_on_bottom_row_clears_that_row_only() {
    let mut c = Console::new();
    c.window(1, 1, 4, 2, test_location()).unwrap();
    c.write(&Value::Str("ABCD".into()), test_location())
        .unwrap();
    c.write(&Value::Str("EFGH".into()), test_location())
        .unwrap();

    c.goto_xy(1, 2, test_location()).unwrap();
    c.del_line(test_location()).unwrap();

    assert_eq!(
        c.test_line_text(1).chars().take(4).collect::<String>(),
        "ABCD"
    );
    assert_eq!(
        c.test_line_text(2).chars().take(4).collect::<String>(),
        "    "
    );
}

#[test]
fn console_ins_line_on_top_row_shifts_rows_down_and_drops_bottom() {
    let mut c = Console::new();
    c.window(1, 1, 4, 2, test_location()).unwrap();
    c.write(&Value::Str("ABCD".into()), test_location())
        .unwrap();
    c.write(&Value::Str("EFGH".into()), test_location())
        .unwrap();

    c.goto_xy(1, 1, test_location()).unwrap();
    c.ins_line(test_location()).unwrap();

    assert_eq!(
        c.test_line_text(1).chars().take(4).collect::<String>(),
        "    "
    );
    assert_eq!(
        c.test_line_text(2).chars().take(4).collect::<String>(),
        "ABCD"
    );
}

#[test]
fn console_set_text_attr_rejects_values_outside_byte_range() {
    let mut c = Console::new();

    let negative = c.set_text_attr(-1, test_location()).unwrap_err();
    assert_eq!(
        negative.message,
        "SetTextAttr expects an attribute from 0 to 255, got -1"
    );

    let overflow = c.set_text_attr(256, test_location()).unwrap_err();
    assert_eq!(
        overflow.message,
        "SetTextAttr expects an attribute from 0 to 255, got 256"
    );
}

#[test]
fn console_video_helpers_are_stable_at_brightness_edges() {
    let mut c = Console::new();

    c.set_text_attr(0x11, test_location()).unwrap();
    c.low_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x11);

    c.high_video(test_location()).unwrap();
    c.high_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x19);
}

#[test]
fn console_cursor_big_forces_visible_block_cursor() {
    let mut c = Console::new();
    c.cursor_off(test_location()).unwrap();
    assert!(!c.state.cursor_visible);

    c.cursor_big(test_location()).unwrap();
    assert!(c.state.cursor_visible);
    assert!(c.state.cursor_big);
}

#[test]
fn console_text_mode_rejects_negative_values() {
    let mut c = Console::new();
    let error = c.text_mode(-1, test_location()).unwrap_err();
    assert_eq!(
        error.message,
        "TextMode expects a non-negative mode value, got -1"
    );
}

#[test]
fn console_text_mode_resets_window_colors_and_screen_contents() {
    let mut c = Console::new();
    c.window(10, 5, 12, 6, test_location()).unwrap();
    c.text_color(12, test_location()).unwrap();
    c.text_background(2, test_location()).unwrap();
    c.write(&Value::Str("XYZ".into()), test_location()).unwrap();

    c.text_mode(7, test_location()).unwrap();

    assert_eq!(c.last_mode(), 7);
    assert_eq!(c.wind_min(), 0x0101);
    assert_eq!(c.wind_max(), 0x1950);
    assert_eq!(c.text_attr(), 0x07);
    assert_eq!(c.test_cell(10, 5), (' ', 7, 0));
}

#[test]
fn console_text_mode_tracks_last_mode_and_resets_cursor() {
    let mut c = Console::new();
    c.window(10, 5, 12, 6, test_location()).unwrap();
    c.goto_xy(2, 2, test_location()).unwrap();
    c.text_mode(3, test_location()).unwrap();
    assert_eq!(c.last_mode(), 3);
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
    assert_eq!(c.screen_width(), 80);
    assert_eq!(c.screen_height(), 25);
}

#[test]
fn console_resize_preserves_overlapping_cells() {
    let mut c = Console::new();
    c.write(&Value::Str("abc".into()), test_location()).unwrap();
    c.resize(2, 1);
    assert_eq!(c.screen_width(), 2);
    assert_eq!(c.screen_height(), 1);
    assert_eq!(
        c.test_line_text(1).chars().take(2).collect::<String>(),
        "ab"
    );
}

#[test]
fn console_resize_clamps_screen_and_cursor_to_minimum_size() {
    let mut c = Console::new();
    c.goto_xy(5, 4, test_location()).unwrap();
    c.resize(0, 0);

    assert_eq!(c.screen_width(), 1);
    assert_eq!(c.screen_height(), 1);
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
    assert_eq!(c.wind_min(), 0x0101);
    assert_eq!(c.wind_max(), 0x0101);
}

#[test]
fn console_sound_rejects_non_positive_frequencies() {
    let mut c = Console::new();

    let zero = c.sound(0, test_location()).unwrap_err();
    assert_eq!(
        zero.message,
        "Sound expects a positive frequency in Hz, got 0"
    );

    let negative = c.sound(-5, test_location()).unwrap_err();
    assert_eq!(
        negative.message,
        "Sound expects a positive frequency in Hz, got -5"
    );
}

#[test]
fn console_sound_writes_terminal_bell_when_writer_is_attached() {
    let (mut c, bytes) = console_with_shared_writer();

    c.sound(440, test_location()).unwrap();

    assert_eq!(&*bytes.lock().unwrap(), b"\x07");
}

#[test]
fn console_no_sound_and_assign_crt_are_safe_state_operations() {
    let mut c = Console::new();
    assert!(!c.state.crt_mode);

    c.no_sound().unwrap();
    c.assign_crt().unwrap();

    assert!(c.state.crt_mode);
}

#[test]
fn console_session_commands_emit_control_sequences_when_writer_exists() {
    let (mut c, bytes) = console_with_shared_writer();

    c.enter_alt_screen(test_location()).unwrap();
    c.leave_alt_screen(test_location()).unwrap();
    c.enable_mouse(test_location()).unwrap();
    c.disable_mouse(test_location()).unwrap();
    c.enable_focus(test_location()).unwrap();
    c.disable_focus(test_location()).unwrap();
    c.enable_paste(test_location()).unwrap();
    c.disable_paste(test_location()).unwrap();

    assert!(!bytes.lock().unwrap().is_empty());
}

#[test]
fn console_session_commands_are_noops_without_writer() {
    let mut c = Console::new();

    c.enter_alt_screen(test_location()).unwrap();
    c.leave_alt_screen(test_location()).unwrap();
    c.enable_mouse(test_location()).unwrap();
    c.disable_mouse(test_location()).unwrap();
    c.enable_focus(test_location()).unwrap();
    c.disable_focus(test_location()).unwrap();
    c.enable_paste(test_location()).unwrap();
    c.disable_paste(test_location()).unwrap();
}
