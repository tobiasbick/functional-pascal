use super::*;

fn cell(glyph: char, foreground: ConsoleColor) -> ConsoleCell {
    ConsoleCell {
        glyph,
        foreground,
        background: ConsoleColor::Crt(0),
    }
}

#[test]
fn saved_region_restores_glyphs_and_extended_colors_once() {
    let mut console = Console::new();
    let original = cell(
        'A',
        ConsoleColor::Rgb {
            red: 10,
            green: 20,
            blue: 30,
        },
    );
    console.put_cell(1, 1, original, test_location()).unwrap();
    let saved = console
        .save_region(
            ConsoleRect {
                x: 1,
                y: 1,
                width: 1,
                height: 1,
            },
            test_location(),
        )
        .unwrap();
    console
        .put_cell(1, 1, cell('B', ConsoleColor::Ansi256(42)), test_location())
        .unwrap();

    console.restore_region(saved, test_location()).unwrap();

    assert_eq!(console.get_cell(1, 1), Some(original));
    assert!(console.restore_region(saved, test_location()).is_err());
}

#[test]
fn discard_region_expires_without_restoring() {
    let mut console = Console::new();
    let saved = console
        .save_region(
            ConsoleRect {
                x: 1,
                y: 1,
                width: 1,
                height: 1,
            },
            test_location(),
        )
        .unwrap();

    console.discard_region(saved, test_location()).unwrap();

    assert!(console.restore_region(saved, test_location()).is_err());
}

#[test]
fn saved_region_preserves_complete_wide_glyph() {
    let mut console = Console::new();
    let wide = cell('中', ConsoleColor::Crt(14));
    console.put_cell(2, 1, wide, test_location()).unwrap();
    let saved = console
        .save_region(
            ConsoleRect {
                x: 3,
                y: 1,
                width: 1,
                height: 1,
            },
            test_location(),
        )
        .unwrap();
    console
        .put_cell(3, 1, cell('X', ConsoleColor::Crt(7)), test_location())
        .unwrap();

    console.restore_region(saved, test_location()).unwrap();

    assert_eq!(console.get_cell(2, 1), Some(wide));
    assert_eq!(console.get_cell(3, 1), None);
}
