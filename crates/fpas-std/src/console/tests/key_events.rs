use super::*;

fn test_key_input() -> KeyInput {
    let mut input = KeyInput::new();
    input.push_chars("");
    input
}

#[test]
fn key_input_test_queue_readkey_keypressed() {
    let mut k = test_key_input();
    assert!(!k.key_pressed(test_location()).unwrap());
    k.push_chars("a");
    assert!(k.key_pressed(test_location()).unwrap());
    assert_eq!(k.read_key(test_location()).unwrap(), 'a');
    assert!(!k.key_pressed(test_location()).unwrap());
}

#[test]
fn key_input_extended_sequence() {
    let mut k = test_key_input();
    k.push_chars("\0H");
    assert_eq!(k.read_key(test_location()).unwrap(), '\0');
    assert_eq!(k.read_key(test_location()).unwrap(), 'H');
}

#[test]
fn key_input_key_pressed_sees_event_queue_without_char_queue() {
    let mut k = test_key_input();
    assert!(!k.key_pressed(test_location()).unwrap());
    k.push_key_event(ConsoleKeyEvent::new(0, '\0', false, false, false, false));
    assert!(k.key_pressed(test_location()).unwrap());
}

#[test]
fn key_input_read_key_event_fifo() {
    let mut k = test_key_input();
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
    let mut k = test_key_input();
    k.push_key_event(ConsoleKeyEvent::new(5, ' ', false, false, false, false));
    k.push_chars("z");
    assert_eq!(k.read_key(test_location()).unwrap(), 'z');
    let ev = k.read_key_event(test_location()).unwrap();
    assert_eq!(ev.kind, 5);
    assert_eq!(ev.ch, ' ');
}

#[test]
fn key_input_live_queue_feeds_read_key_event() {
    let mut k = test_key_input();
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
    let mut k = test_key_input();
    k.push_console_event(ConsoleEvent::focus_gained());
    assert!(!k.key_pressed(test_location()).unwrap());
    assert!(k.event_pending(test_location()).unwrap());
}

#[test]
fn key_input_read_event_returns_queued_resize() {
    let mut k = test_key_input();
    k.push_console_event(ConsoleEvent::resize(120, 40));
    assert!(k.event_pending(test_location()).unwrap());
    let event = k.read_event(test_location()).unwrap();
    assert_eq!(event.kind, event_kind_index("Resize"));
    assert_eq!(event.width, 120);
    assert_eq!(event.height, 40);
}

#[test]
fn key_input_read_event_preserves_fifo_across_event_kinds() {
    let mut k = test_key_input();
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
    let mut k = test_key_input();
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
    let mut k = test_key_input();
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
fn key_input_coalesces_live_resize_burst_before_key() {
    let mut k = test_key_input();
    assert!(k.push_live_event(Event::Resize(80, 24)));
    assert!(k.push_live_event(Event::Resize(100, 30)));
    assert!(k.push_live_event(Event::Resize(120, 40)));
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::NONE,
    ))));

    let resize = k.read_event_timeout(0, test_location()).unwrap().unwrap();
    assert_eq!(resize.kind, event_kind_index("Resize"));
    assert_eq!((resize.width, resize.height), (120, 40));

    let key = k.read_event_timeout(0, test_location()).unwrap().unwrap();
    assert_eq!(key.kind, event_kind_index("Key"));
    assert_eq!(key.key.ch, 'x');
    assert!(!k.event_pending(test_location()).unwrap());
}

#[test]
fn key_input_read_event_of_live_key_clears_key_pressed() {
    let mut k = test_key_input();
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('a'),
        KeyModifiers::NONE,
    ))));
    assert!(k.key_pressed(test_location()).unwrap());
    assert!(k.event_pending(test_location()).unwrap());

    let event = k.read_event(test_location()).unwrap();
    assert_eq!(event.kind, event_kind_index("Key"));
    assert_eq!(event.key.ch, 'a');

    assert!(!k.key_pressed(test_location()).unwrap());
    assert!(!k.event_pending(test_location()).unwrap());
}

#[test]
fn key_input_read_key_event_of_live_key_clears_event_pending() {
    let mut k = test_key_input();
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('b'),
        KeyModifiers::CONTROL,
    ))));
    assert!(k.key_pressed(test_location()).unwrap());
    assert!(k.event_pending(test_location()).unwrap());

    let ev = k.read_key_event(test_location()).unwrap();
    assert_eq!(ev.ch, 'b');
    assert!(ev.ctrl);

    assert!(!k.key_pressed(test_location()).unwrap());
    assert!(!k.event_pending(test_location()).unwrap());
}

#[test]
fn key_input_read_key_leaves_intervening_mouse_for_read_event() {
    let mut k = test_key_input();
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::NONE,
    ))));
    assert!(k.push_live_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    })));
    assert!(k.push_live_event(Event::Key(CrosstermKeyEvent::new(
        KeyCode::Char('y'),
        KeyModifiers::NONE,
    ))));

    assert_eq!(k.read_key(test_location()).unwrap(), 'x');
    assert!(k.event_pending(test_location()).unwrap());
    let mouse = k.read_event(test_location()).unwrap();
    assert_eq!(mouse.kind, event_kind_index("Mouse"));
    let second_key = k.read_event(test_location()).unwrap();
    assert_eq!(second_key.kind, event_kind_index("Key"));
    assert_eq!(second_key.key.ch, 'y');
    assert!(!k.key_pressed(test_location()).unwrap());
    assert!(!k.event_pending(test_location()).unwrap());
}

#[test]
fn key_input_event_pending_is_false_when_all_queues_are_empty() {
    let mut k = test_key_input();
    assert!(!k.event_pending(test_location()).unwrap());
}
