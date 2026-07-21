use super::*;

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
