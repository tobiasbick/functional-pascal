use super::*;
use crate::{CommandId, Console, DamageRegion, ViewRect};
use fpas_bytecode::SourceLocation;

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
    console.assign_crt().expect("CRT assignment should succeed");
    console.begin_tui_paint(DamageRegion::FullFrame);
    console
}

#[test]
fn label_paints_text_and_accelerator() {
    let mut console = console();
    LabelWidget::new("Open", Some('O'), LabelStyle::default()).paint(
        &mut console,
        rect(1, 1, 8, 1),
        DamageRegion::FullFrame,
    );

    console
        .finish_tui_paint(loc())
        .expect("paint should finish");
    assert_eq!(console.test_cell(2, 2), ('O', 4, 7));
    assert_eq!(console.test_cell(3, 2), ('p', 0, 7));
    assert_eq!(console.test_cell(9, 2), (' ', 0, 7));
}

#[test]
fn disabled_label_uses_disabled_color_without_accelerator() {
    let mut console = console();
    let mut label = LabelWidget::new("Open", Some('O'), LabelStyle::default());
    label.enabled = false;
    label.paint(&mut console, rect(0, 0, 6, 1), DamageRegion::FullFrame);

    console
        .finish_tui_paint(loc())
        .expect("paint should finish");
    assert_eq!(console.test_cell(1, 1), ('O', 8, 7));
    assert_eq!(console.test_cell(2, 1), ('p', 8, 7));
}

#[test]
fn label_respects_damage_region() {
    let mut console = console();
    LabelWidget::new("Name", Some('N'), LabelStyle::default()).paint(
        &mut console,
        rect(0, 0, 8, 1),
        DamageRegion::Rect(rect(2, 0, 2, 1)),
    );

    console
        .finish_tui_paint(loc())
        .expect("paint should finish");
    assert_eq!(console.test_cell(1, 1), (' ', 7, 0));
    assert_eq!(console.test_cell(3, 1), ('m', 0, 7));
    assert_eq!(console.test_cell(4, 1), ('e', 0, 7));
}

#[test]
fn button_paints_centered_caption_and_tracks_command() {
    let mut console = console();
    let button = ButtonWidget::new("OK", Some(CommandId(10)), ButtonStyle::default());
    button.paint(&mut console, rect(0, 0, 10, 1), DamageRegion::FullFrame);

    console
        .finish_tui_paint(loc())
        .expect("paint should finish");
    assert_eq!(button.command_id, Some(CommandId(10)));
    assert_eq!(button.minimum_width(), 6);
    assert_eq!(console.test_cell(3, 1), ('[', 0, 7));
    assert_eq!(console.test_cell(5, 1), ('O', 0, 7));
    assert_eq!(console.test_cell(8, 1), (']', 0, 7));
}

#[test]
fn default_button_uses_active_style_and_angle_chrome() {
    let mut console = console();
    let mut button = ButtonWidget::new("OK", Some(CommandId(1)), ButtonStyle::default());
    button.default = true;
    button.paint(&mut console, rect(0, 0, 8, 1), DamageRegion::FullFrame);

    console
        .finish_tui_paint(loc())
        .expect("paint should finish");
    assert_eq!(button.minimum_width(), 6);
    assert_eq!(console.test_cell(2, 1), ('<', 15, 0));
    assert_eq!(console.test_cell(4, 1), ('O', 15, 0));
    assert_eq!(console.test_cell(7, 1), ('>', 15, 0));
}

#[test]
fn disabled_button_uses_disabled_color() {
    let mut console = console();
    let mut button = ButtonWidget::new("Cancel", Some(CommandId(2)), ButtonStyle::default());
    button.enabled = false;
    button.paint(&mut console, rect(0, 0, 10, 1), DamageRegion::FullFrame);

    console
        .finish_tui_paint(loc())
        .expect("paint should finish");
    assert_eq!(console.test_cell(1, 1), ('[', 8, 7));
    assert_eq!(console.test_cell(3, 1), ('C', 8, 7));
}
