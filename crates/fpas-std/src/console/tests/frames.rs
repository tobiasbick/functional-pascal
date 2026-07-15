use super::*;

fn cell(glyph: char) -> ConsoleCell {
    ConsoleCell {
        glyph,
        foreground: ConsoleColor::Crt(7),
        background: ConsoleColor::Crt(0),
    }
}

fn written_len(bytes: &Arc<Mutex<Vec<u8>>>) -> usize {
    bytes.lock().unwrap().len()
}

#[test]
fn begin_frame_defers_terminal_output_until_present() {
    let (mut console, bytes) = console_with_shared_writer();

    console.begin_frame();
    console.put_cell(1, 1, cell('X'), test_location()).unwrap();
    assert_eq!(written_len(&bytes), 0);

    console.present(test_location()).unwrap();
    assert!(written_len(&bytes) > 0);
}

#[test]
fn nested_frames_only_flush_at_outermost_present() {
    let (mut console, bytes) = console_with_shared_writer();

    console.begin_frame();
    console.begin_frame();
    console.put_cell(1, 1, cell('X'), test_location()).unwrap();
    console.present(test_location()).unwrap();
    assert_eq!(written_len(&bytes), 0);

    console.present(test_location()).unwrap();
    assert!(written_len(&bytes) > 0);
}

#[test]
fn put_cell_remains_immediate_outside_a_frame() {
    let (mut console, bytes) = console_with_shared_writer();

    console.put_cell(1, 1, cell('X'), test_location()).unwrap();

    assert!(written_len(&bytes) > 0);
}
