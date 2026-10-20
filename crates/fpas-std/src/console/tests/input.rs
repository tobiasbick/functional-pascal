use super::*;

#[test]
fn text_input_read_then_readln_shares_buffer() {
    let mut t = TextInput::new();
    t.push_line("ab");
    assert_eq!(t.read_char(test_location()).unwrap(), Some('a'));
    assert_eq!(t.read_char(test_location()).unwrap(), Some('b'));
    assert_eq!(t.read_line(test_location()).unwrap().as_deref(), Some(""));
}

#[test]
fn text_input_readln_then_read() {
    let mut t = TextInput::new();
    t.push_line("xy");
    assert_eq!(t.read_line(test_location()).unwrap().as_deref(), Some("xy"));
    t.push_line("z");
    assert_eq!(t.read_char(test_location()).unwrap(), Some('z'));
    assert_eq!(t.read_line(test_location()).unwrap().as_deref(), Some(""));
}

#[test]
fn debugger_text_input_never_reads_os_stdin() {
    let mut input = TextInput::without_os_stdin();
    let error = input
        .read_line(test_location())
        .expect_err("debugger input must not read process stdin");
    assert!(error.message.contains("no input available"));
    input.push_line("queued");
    assert_eq!(
        input.read_line(test_location()).unwrap().as_deref(),
        Some("queued")
    );
}

#[test]
fn debugger_text_input_eof_is_end_of_input() {
    let mut input = TextInput::without_os_stdin();
    input.close_input();
    assert_eq!(input.read_line(test_location()).unwrap(), None);
    assert_eq!(input.read_char(test_location()).unwrap(), None);
    input.push_line("late");
    assert_eq!(
        input.read_line(test_location()).unwrap().as_deref(),
        Some("late")
    );
}

#[test]
fn debugger_text_input_clear_drops_unread_lines() {
    let mut input = TextInput::without_os_stdin();
    input.push_line("secret");
    input.clear_queued();
    let error = input
        .read_line(test_location())
        .expect_err("cleared debugger input must discard unread lines");
    assert!(error.message.contains("no input available"));
}

#[test]
fn closed_input_drains_pending_characters_and_queued_empty_lines() {
    let mut input = TextInput::without_os_stdin();
    input.push_line("€rest");
    input.push_line("");
    assert_eq!(input.read_char(test_location()).unwrap(), Some('€'));
    input.close_input();
    assert_eq!(
        input.read_line(test_location()).unwrap().as_deref(),
        Some("rest")
    );
    assert_eq!(
        input.read_line(test_location()).unwrap().as_deref(),
        Some("")
    );
    assert_eq!(input.read_char(test_location()).unwrap(), None);
    assert_eq!(input.read_line(test_location()).unwrap(), None);
    input.clear_queued();
    assert_eq!(input.read_line(test_location()).unwrap(), None);
}

#[test]
fn stdin_reader_keeps_blank_lines_and_final_unterminated_data() {
    use crate::console::input::read_line_from_reader;
    let mut reader = std::io::Cursor::new("\n\r\n€tail\r");
    for expected in [Some("\n"), Some("\n"), Some("€tail\r"), None, None] {
        assert_eq!(
            read_line_from_reader(&mut reader, test_location())
                .unwrap()
                .as_deref(),
            expected
        );
    }
}

#[test]
fn invalid_stdin_utf8_is_an_input_error_rather_than_eof() {
    let mut reader = std::io::Cursor::new([0xff]);
    let error = crate::console::input::read_line_from_reader(&mut reader, test_location())
        .expect_err("invalid UTF-8 is not end of input");
    assert!(error.message.contains("ReadLn failed"));
}
