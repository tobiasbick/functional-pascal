use super::*;

#[test]
fn text_input_read_then_readln_shares_buffer() {
    let mut t = TextInput::new();
    t.push_line("ab");
    assert_eq!(t.read_char(test_location()).unwrap(), 'a');
    assert_eq!(t.read_char(test_location()).unwrap(), 'b');
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
fn debugger_text_input_never_reads_os_stdin() {
    let mut input = TextInput::without_os_stdin();
    let error = input
        .read_line(test_location())
        .expect_err("debugger input must not read process stdin");
    assert!(error.message.contains("no input available"));
    input.push_line("queued");
    assert_eq!(input.read_line(test_location()).unwrap(), "queued");
}

#[test]
fn debugger_text_input_eof_is_end_of_input() {
    let mut input = TextInput::without_os_stdin();
    input.close_input();
    let error = input
        .read_line(test_location())
        .expect_err("closed debugger input must report end of input");
    assert!(error.message.contains("end of input"));
    input.push_line("late");
    assert_eq!(input.read_line(test_location()).unwrap(), "late");
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
