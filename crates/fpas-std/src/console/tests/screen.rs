use super::*;

#[test]
fn console_window_coordinates_are_relative() {
    let mut c = Console::new();
    c.window(10, 5, 12, 6, test_location()).unwrap();
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
    c.goto_xy(2, 2, test_location()).unwrap();
    assert_eq!(c.where_x(), 2);
    assert_eq!(c.where_y(), 2);
    c.write(&Value::Str("X".into()), test_location()).unwrap();
    assert_eq!(c.test_line_text(6).chars().nth(10), Some('X'));
}

#[test]
fn console_clrscr_only_clears_active_window() {
    let mut c = Console::new();
    c.write(&Value::Str("ABCDE".into()), test_location())
        .unwrap();
    c.window(2, 1, 4, 1, test_location()).unwrap();
    c.clr_scr(test_location()).unwrap();
    let row = c.test_line_text(1);
    assert_eq!(row.chars().take(5).collect::<String>(), "A   E");
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
}

#[test]
fn console_scrolls_inside_active_window() {
    let mut c = Console::new();
    c.window(1, 1, 3, 2, test_location()).unwrap();
    c.write(&Value::Str("ab".into()), test_location()).unwrap();
    c.write(&Value::Str("c".into()), test_location()).unwrap();
    c.write(&Value::Str("de".into()), test_location()).unwrap();
    c.write(&Value::Str("fg".into()), test_location()).unwrap();
    assert_eq!(
        c.test_line_text(1).chars().take(3).collect::<String>(),
        "def"
    );
    assert_eq!(
        c.test_line_text(2).chars().take(3).collect::<String>(),
        "g  "
    );
    assert_eq!(c.where_x(), 2);
    assert_eq!(c.where_y(), 2);
}

#[test]
fn console_del_line_and_ins_line_inside_window() {
    let mut c = Console::new();
    c.window(1, 1, 5, 3, test_location()).unwrap();
    c.write(&Value::Str("AAAAA".into()), test_location())
        .unwrap();
    c.goto_xy(1, 2, test_location()).unwrap();
    c.write(&Value::Str("BBBBB".into()), test_location())
        .unwrap();
    c.goto_xy(1, 3, test_location()).unwrap();
    c.write(&Value::Str("CCCCC".into()), test_location())
        .unwrap();

    c.goto_xy(1, 2, test_location()).unwrap();
    c.del_line(test_location()).unwrap();
    assert_eq!(
        c.test_line_text(2).chars().take(5).collect::<String>(),
        "CCCCC"
    );
    assert_eq!(
        c.test_line_text(3).chars().take(5).collect::<String>(),
        "     "
    );

    c.goto_xy(1, 2, test_location()).unwrap();
    c.ins_line(test_location()).unwrap();
    assert_eq!(
        c.test_line_text(2).chars().take(5).collect::<String>(),
        "     "
    );
    assert_eq!(
        c.test_line_text(3).chars().take(5).collect::<String>(),
        "CCCCC"
    );
}

#[test]
fn console_wind_min_and_wind_max_follow_window_and_resize() {
    let mut c = Console::new();
    assert_eq!(c.wind_min(), 0x0101);
    assert_eq!(c.wind_max(), 0x1950);

    c.window(10, 5, 20, 8, test_location()).unwrap();
    assert_eq!(c.wind_min(), 0x050A);
    assert_eq!(c.wind_max(), 0x0814);

    c.resize(12, 6);
    assert_eq!(c.wind_min(), 0x050A);
    assert_eq!(c.wind_max(), 0x060C);
}

#[test]
fn console_del_line_on_bottom_row_clears_that_row_only() {
    let mut c = Console::new();
    c.window(1, 1, 4, 2, test_location()).unwrap();
    c.write(&Value::Str("ABCD".into()), test_location())
        .unwrap();
    c.write(&Value::Str("EFGH".into()), test_location())
        .unwrap();

    c.goto_xy(1, 2, test_location()).unwrap();
    c.del_line(test_location()).unwrap();

    assert_eq!(
        c.test_line_text(1).chars().take(4).collect::<String>(),
        "ABCD"
    );
    assert_eq!(
        c.test_line_text(2).chars().take(4).collect::<String>(),
        "    "
    );
}

#[test]
fn console_ins_line_on_top_row_shifts_rows_down_and_drops_bottom() {
    let mut c = Console::new();
    c.window(1, 1, 4, 2, test_location()).unwrap();
    c.write(&Value::Str("ABCD".into()), test_location())
        .unwrap();
    c.write(&Value::Str("EFGH".into()), test_location())
        .unwrap();

    c.goto_xy(1, 1, test_location()).unwrap();
    c.ins_line(test_location()).unwrap();

    assert_eq!(
        c.test_line_text(1).chars().take(4).collect::<String>(),
        "    "
    );
    assert_eq!(
        c.test_line_text(2).chars().take(4).collect::<String>(),
        "ABCD"
    );
}

#[test]
fn console_cursor_big_forces_visible_block_cursor() {
    let mut c = Console::new();
    c.cursor_off(test_location()).unwrap();
    assert!(!c.state.cursor_visible);

    c.cursor_big(test_location()).unwrap();
    assert!(c.state.cursor_visible);
    assert!(c.state.cursor_big);
}

#[test]
fn console_text_mode_rejects_negative_values() {
    let mut c = Console::new();
    let error = c.text_mode(-1, test_location()).unwrap_err();
    assert_eq!(
        error.message,
        "TextMode expects a non-negative mode value, got -1"
    );
}

#[test]
fn console_text_mode_resets_window_colors_and_screen_contents() {
    let mut c = Console::new();
    c.window(10, 5, 12, 6, test_location()).unwrap();
    c.text_color(12, test_location()).unwrap();
    c.text_background(2, test_location()).unwrap();
    c.write(&Value::Str("XYZ".into()), test_location()).unwrap();

    c.text_mode(7, test_location()).unwrap();

    assert_eq!(c.last_mode(), 7);
    assert_eq!(c.wind_min(), 0x0101);
    assert_eq!(c.wind_max(), 0x1950);
    assert_eq!(c.text_attr(), 0x07);
    assert_eq!(c.test_cell(10, 5), (' ', 7, 0));
}

#[test]
fn console_text_mode_tracks_last_mode_and_resets_cursor() {
    let mut c = Console::new();
    c.window(10, 5, 12, 6, test_location()).unwrap();
    c.goto_xy(2, 2, test_location()).unwrap();
    c.text_mode(3, test_location()).unwrap();
    assert_eq!(c.last_mode(), 3);
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
    assert_eq!(c.screen_width(), 80);
    assert_eq!(c.screen_height(), 25);
}

#[test]
fn console_resize_preserves_overlapping_cells() {
    let mut c = Console::new();
    c.write(&Value::Str("abc".into()), test_location()).unwrap();
    c.resize(2, 1);
    assert_eq!(c.screen_width(), 2);
    assert_eq!(c.screen_height(), 1);
    assert_eq!(
        c.test_line_text(1).chars().take(2).collect::<String>(),
        "ab"
    );
}

#[test]
fn console_resize_clamps_screen_and_cursor_to_minimum_size() {
    let mut c = Console::new();
    c.goto_xy(5, 4, test_location()).unwrap();
    c.resize(0, 0);

    assert_eq!(c.screen_width(), 1);
    assert_eq!(c.screen_height(), 1);
    assert_eq!(c.where_x(), 1);
    assert_eq!(c.where_y(), 1);
    assert_eq!(c.wind_min(), 0x0101);
    assert_eq!(c.wind_max(), 0x0101);
}
