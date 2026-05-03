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
