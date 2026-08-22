use super::*;

#[test]
fn console_sound_rejects_non_positive_frequencies() {
    let mut c = Console::new();

    let zero = c
        .sound(0, test_location())
        .expect_err("Sound must reject zero hertz");
    assert_eq!(
        zero.message,
        "Sound expects a positive frequency in Hz, got 0"
    );

    let negative = c
        .sound(-5, test_location())
        .expect_err("Sound must reject negative frequencies");
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
