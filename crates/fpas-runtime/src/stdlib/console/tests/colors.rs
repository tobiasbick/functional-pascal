use super::*;

#[test]
fn console_text_attributes_and_clreol_use_current_colors() {
    let mut c = Console::new();
    c.text_color(12, test_location()).unwrap();
    c.text_background(1, test_location()).unwrap();
    c.write(&Value::Str("xy".into()), test_location()).unwrap();
    c.clr_eol(test_location()).unwrap();
    let first = c.test_cell(1, 1);
    let cleared = c.test_cell(10, 1);
    assert_eq!(first.0, 'x');
    assert_eq!(first.1, 12);
    assert_eq!(first.2, 1);
    assert_eq!(cleared.0, ' ');
    assert_eq!(cleared.1, 12);
    assert_eq!(cleared.2, 1);
}

#[test]
fn console_text_attr_and_video_helpers() {
    let mut c = Console::new();
    c.set_text_attr(0x1E, test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x1E);

    c.low_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x16);

    c.high_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x1E);

    c.norm_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x07);
}

#[test]
fn console_set_text_attr_rejects_values_outside_byte_range() {
    let mut c = Console::new();

    let negative = c
        .set_text_attr(-1, test_location())
        .expect_err("SetTextAttr must reject negative attributes");
    assert_eq!(
        negative.message,
        "SetTextAttr expects an attribute from 0 to 255, got -1"
    );

    let overflow = c
        .set_text_attr(256, test_location())
        .expect_err("SetTextAttr must reject attributes above one byte");
    assert_eq!(
        overflow.message,
        "SetTextAttr expects an attribute from 0 to 255, got 256"
    );
}

#[test]
fn console_video_helpers_are_stable_at_brightness_edges() {
    let mut c = Console::new();

    c.set_text_attr(0x11, test_location()).unwrap();
    c.low_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x11);

    c.high_video(test_location()).unwrap();
    c.high_video(test_location()).unwrap();
    assert_eq!(c.text_attr(), 0x19);
}

#[test]
fn console_extended_colors_are_stored_in_screen_cells() {
    let mut c = Console::new();

    c.text_color_rgb(255, 128, 0, test_location()).unwrap();
    c.text_background_256(196, test_location()).unwrap();
    c.write(&Value::Str("X".into()), test_location()).unwrap();

    assert_eq!(c.text_attr(), 0x07);
    assert_eq!(
        c.test_cell_colors(1, 1),
        ('X', "rgb:255,128,0".to_string(), "ansi256:196".to_string())
    );
}

#[test]
fn console_packed_color_calls_reset_extended_color_path() {
    let mut c = Console::new();

    c.text_color_rgb(255, 128, 0, test_location()).unwrap();
    c.text_background_rgb(0, 0, 64, test_location()).unwrap();
    c.set_text_attr(0x1E, test_location()).unwrap();
    c.write(&Value::Str("Y".into()), test_location()).unwrap();

    assert_eq!(c.test_cell(1, 1), ('Y', 14, 1));
}

#[test]
fn console_redraw_emits_extended_colors_from_screen_buffer() {
    let (mut c, bytes) = console_with_shared_writer();

    c.text_color_rgb(255, 128, 0, test_location()).unwrap();
    c.text_background_256(196, test_location()).unwrap();
    c.write(&Value::Str("X".into()), test_location()).unwrap();
    bytes.lock().unwrap().clear();

    c.clr_eol(test_location()).unwrap();

    let rendered = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    assert!(rendered.contains("38;2;255;128;0"), "{rendered:?}");
    assert!(rendered.contains("48;5;196"), "{rendered:?}");
}

#[test]
fn console_resize_preserves_extended_color_cells() {
    let mut c = Console::new();

    c.text_color_rgb(10, 20, 30, test_location()).unwrap();
    c.text_background_rgb(40, 50, 60, test_location()).unwrap();
    c.write(&Value::Str("R".into()), test_location()).unwrap();
    c.resize(120, 40);

    assert_eq!(
        c.test_cell_colors(1, 1),
        ('R', "rgb:10,20,30".to_string(), "rgb:40,50,60".to_string())
    );
}

#[test]
fn console_text_mode_resets_extended_color_path_to_packed_defaults() {
    let mut c = Console::new();

    c.text_color_rgb(255, 128, 0, test_location()).unwrap();
    c.text_background_256(196, test_location()).unwrap();
    c.text_mode(3, test_location()).unwrap();
    c.write(&Value::Str("Z".into()), test_location()).unwrap();

    assert_eq!(c.test_cell(1, 1), ('Z', 7, 0));
}
