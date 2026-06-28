use super::*;
use crate::{Console, DamageRegion, ViewRect};
use fpas_bytecode::SourceLocation;

const ACTIVE_FG: u8 = 15;
const ACTIVE_BG: u8 = 0;

fn rect(x: i64, y: i64, width: i64, height: i64) -> ViewRect {
    ViewRect {
        x,
        y,
        width,
        height,
    }
}

fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn console() -> Console {
    let mut console = Console::new();
    let assigned = console.assign_crt();
    assert!(
        assigned.is_ok(),
        "CRT assignment should succeed: {assigned:?}"
    );
    console.begin_tui_paint(DamageRegion::FullFrame);
    console
}

fn finish_paint(console: &mut Console) {
    let finished = console.finish_tui_paint(loc());
    assert!(finished.is_ok(), "paint should finish: {finished:?}");
}

fn assert_active_cell(cell: (char, u8, u8), ch: char) {
    assert_eq!(cell, (ch, ACTIVE_FG, ACTIVE_BG));
}

#[test]
fn focus_style_defaults_keep_shared_active_palette() {
    let button = ButtonStyle::default();
    assert_eq!((button.active_fg, button.active_bg), (ACTIVE_FG, ACTIVE_BG));

    let checkbox = CheckBoxStyle::default();
    assert_eq!(
        (checkbox.active_fg, checkbox.active_bg),
        (ACTIVE_FG, ACTIVE_BG)
    );

    let input = InputLineStyle::default();
    assert_eq!((input.cursor_fg, input.cursor_bg), (ACTIVE_FG, ACTIVE_BG));

    let list_box = ListBoxStyle::default();
    assert_eq!(
        (list_box.active_fg, list_box.active_bg),
        (ACTIVE_FG, ACTIVE_BG)
    );

    let memo = MemoStyle::default();
    assert_eq!((memo.cursor_fg, memo.cursor_bg), (ACTIVE_FG, ACTIVE_BG));

    let radio = RadioGroupStyle::default();
    assert_eq!((radio.active_fg, radio.active_bg), (ACTIVE_FG, ACTIVE_BG));
}

#[test]
fn focused_controls_paint_the_shared_active_palette() {
    let mut button_console = console();
    let mut button = ButtonWidget::new("OK", None, ButtonStyle::default());
    button.focused = true;
    button.paint(
        &mut button_console,
        rect(0, 0, 6, 1),
        DamageRegion::FullFrame,
    );
    finish_paint(&mut button_console);
    assert_active_cell(button_console.test_cell(1, 1), '[');

    let mut checkbox_console = console();
    let mut checkbox = CheckBoxWidget::new("Sync", None, None, CheckBoxStyle::default());
    checkbox.focused = true;
    checkbox.paint(
        &mut checkbox_console,
        rect(0, 0, 8, 1),
        DamageRegion::FullFrame,
    );
    finish_paint(&mut checkbox_console);
    assert_active_cell(checkbox_console.test_cell(1, 1), '[');

    let mut input_console = console();
    let mut input = InputLineWidget::new("abc", InputLineStyle::default());
    input.focused = true;
    input.set_cursor(1);
    input.paint(
        &mut input_console,
        rect(0, 0, 5, 1),
        DamageRegion::FullFrame,
    );
    finish_paint(&mut input_console);
    assert_active_cell(input_console.test_cell(2, 1), 'b');

    let mut list_console = console();
    let mut list = ListBoxWidget::new(
        vec![ListBoxItem {
            text: "One".to_string(),
            command_id: None,
            enabled: true,
        }],
        1,
    );
    list.focused = true;
    list.paint(&mut list_console, rect(0, 0, 4, 1), DamageRegion::FullFrame);
    finish_paint(&mut list_console);
    assert_active_cell(list_console.test_cell(1, 1), 'O');

    let mut memo_console = console();
    let mut memo = MemoWidget::new("", 1);
    memo.focused = true;
    memo.paint(&mut memo_console, rect(0, 0, 5, 1), DamageRegion::FullFrame);
    finish_paint(&mut memo_console);
    assert_active_cell(memo_console.test_cell(1, 1), ' ');

    let mut radio_console = console();
    let mut radio = RadioGroupWidget::new(
        vec![RadioOption::new("One", None, None)],
        RadioGroupStyle::default(),
    );
    radio.focused = true;
    radio.paint(
        &mut radio_console,
        rect(0, 0, 8, 1),
        DamageRegion::FullFrame,
    );
    finish_paint(&mut radio_console);
    assert_active_cell(radio_console.test_cell(1, 1), '(');
}
