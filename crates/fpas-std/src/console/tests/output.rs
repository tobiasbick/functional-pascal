use super::*;

#[test]
fn headless_console_captures_fragments_as_one_complete_line() {
    let mut console = Console::new();

    console
        .write(&Value::Str("captured ".into()), test_location())
        .unwrap();
    console
        .write_ln(&Value::Str("line".into()), test_location())
        .unwrap();

    assert_eq!(console.output().lines, ["captured line"]);
}

#[test]
fn writer_console_streams_complete_lines_without_capturing_them() {
    let (mut console, bytes) = console_with_shared_writer();

    console
        .write_ln(&Value::Str("streamed".into()), test_location())
        .unwrap();

    assert_eq!(
        (
            bytes.lock().unwrap().clone(),
            console.output().lines.clone()
        ),
        (b"streamed\n".to_vec(), Vec::<String>::new())
    );
}

#[test]
fn writer_console_does_not_retain_an_unfinished_line() {
    let (mut console, _) = console_with_shared_writer();

    console
        .write(&Value::Str("unfinished".into()), test_location())
        .unwrap();

    assert!(console.capture_line_buf.is_empty());
}
