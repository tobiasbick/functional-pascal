use super::*;

fn crt_cell(glyph: char, foreground: u8, background: u8) -> ConsoleCell {
    ConsoleCell {
        glyph,
        foreground: ConsoleColor::Crt(foreground),
        background: ConsoleColor::Crt(background),
    }
}

#[test]
fn put_and_get_cell_round_trip_all_color_kinds() {
    let mut console = Console::new();
    let cell = ConsoleCell {
        glyph: 'X',
        foreground: ConsoleColor::Rgb {
            red: 12,
            green: 34,
            blue: 56,
        },
        background: ConsoleColor::Ansi256(123),
    };

    console.put_cell(2, 3, cell, test_location()).unwrap();

    assert_eq!(console.get_cell(2, 3), Some(cell));
}

#[test]
fn fill_rect_clips_to_screen_bounds() {
    let mut console = Console::new();
    console.resize(4, 2);

    console
        .fill_rect(
            ConsoleRect {
                x: 3,
                y: 1,
                width: 5,
                height: 5,
            },
            crt_cell('#', 14, 1),
            test_location(),
        )
        .unwrap();

    assert_eq!(console.test_line_text(1), "  ##");
    assert_eq!(console.test_line_text(2), "  ##");
}

#[test]
fn write_cells_advances_by_unicode_display_width() {
    let mut console = Console::new();
    let cells = [
        crt_cell('A', 7, 0),
        crt_cell('中', 10, 0),
        crt_cell('B', 12, 0),
    ];

    console.write_cells(1, 1, &cells, test_location()).unwrap();

    assert_eq!(
        console
            .test_line_text(1)
            .chars()
            .take(4)
            .collect::<String>(),
        "A中 B"
    );
    assert_eq!(console.get_cell(2, 1), Some(cells[1]));
    assert_eq!(console.get_cell(3, 1), None);
    assert_eq!(Console::display_width("A中B"), 4);
}

#[test]
fn overwriting_wide_continuation_repairs_both_cells() {
    let mut console = Console::new();
    console
        .put_cell(2, 1, crt_cell('中', 7, 0), test_location())
        .unwrap();

    console
        .put_cell(3, 1, crt_cell('X', 14, 0), test_location())
        .unwrap();

    assert_eq!(console.test_line_text(1).chars().nth(1), Some(' '));
    assert_eq!(console.test_line_text(1).chars().nth(2), Some('X'));
}

#[test]
fn resize_does_not_leave_a_wide_glyph_at_the_last_column() {
    let mut console = Console::new();
    console
        .put_cell(2, 1, crt_cell('中', 7, 0), test_location())
        .unwrap();

    console.resize(2, 1);

    assert_eq!(console.get_cell(2, 1), Some(crt_cell(' ', 7, 0)));
}

#[test]
fn cell_operations_reject_standalone_zero_width_glyphs() {
    let mut console = Console::new();

    let error = console
        .put_cell(1, 1, crt_cell('\u{0301}', 7, 0), test_location())
        .unwrap_err();

    assert_eq!(
        error.message,
        "PutCell cannot paint a standalone zero-width glyph"
    );
}
