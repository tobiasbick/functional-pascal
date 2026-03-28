use super::*;

#[test]
fn std_console_window_invalid_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  Window(0, 1, 10, 10)
end.",
    );
    assert!(
        msg.contains("outside the screen") || msg.contains("Window"),
        "{msg}"
    );
}

#[test]
fn std_console_gotoxy_invalid_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  Window(5, 5, 6, 6);
  GotoXY(3, 1)
end.",
    );
    assert!(
        msg.contains("active window") || msg.contains("coordinate"),
        "{msg}"
    );
}

#[test]
fn std_console_textcolor_invalid_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  TextColor(16)
end.",
    );
    assert!(
        msg.contains("0 to 15") || msg.contains("TextColor"),
        "{msg}"
    );
}

#[test]
fn std_console_delay_negative_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  Delay(-1)
end.",
    );
    assert!(
        msg.contains("non-negative") || msg.contains("Delay"),
        "{msg}"
    );
}

#[test]
fn std_console_set_text_attr_invalid_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  SetTextAttr(300)
end.",
    );
    assert!(
        msg.contains("0 to 255") || msg.contains("SetTextAttr"),
        "{msg}"
    );
}

#[test]
fn std_console_text_mode_negative_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  TextMode(-1)
end.",
    );
    assert!(
        msg.contains("non-negative") || msg.contains("TextMode"),
        "{msg}"
    );
}

#[test]
fn std_console_sound_non_positive_runtime() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  Sound(0)
end.",
    );
    assert!(
        msg.contains("positive frequency") || msg.contains("Sound"),
        "{msg}"
    );
}
