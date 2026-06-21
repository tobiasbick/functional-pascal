use super::*;

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

#[test]
fn console_tui_paint_defers_terminal_output_until_finish() {
    let (mut c, bytes) = console_with_shared_writer();

    c.assign_crt().unwrap();
    c.write(&Value::Str("A".into()), test_location()).unwrap();
    bytes.lock().unwrap().clear();

    c.begin_tui_paint(crate::DamageRegion::Rect(crate::ViewRect {
        x: 1,
        y: 0,
        width: 1,
        height: 1,
    }));
    c.text_color_rgb(255, 64, 0, test_location()).unwrap();
    c.write(&Value::Str("B".into()), test_location()).unwrap();

    assert!(
        bytes.lock().unwrap().is_empty(),
        "CRT output should stay buffered until the hosted paint finishes"
    );

    c.finish_tui_paint(test_location()).unwrap();

    let output = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(output.contains('B'));
    assert!(!output.contains("\x1b[2J"));
}

#[test]
fn console_tui_paint_unions_host_damage_with_actual_console_mutations() {
    let (mut c, bytes) = console_with_shared_writer();

    c.assign_crt().unwrap();
    c.write(&Value::Str("A".into()), test_location()).unwrap();
    bytes.lock().unwrap().clear();

    c.begin_tui_paint(crate::DamageRegion::Rect(crate::ViewRect {
        x: 10,
        y: 10,
        width: 1,
        height: 1,
    }));
    c.goto_xy(1, 1, test_location()).unwrap();
    c.write(&Value::Str("Z".into()), test_location()).unwrap();
    c.finish_tui_paint(test_location()).unwrap();

    let output = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(
        output.contains('Z'),
        "present should include cells mutated during the deferred paint even when the host damage hint points elsewhere"
    );
    assert!(!output.contains("\x1b[2J"));
}
