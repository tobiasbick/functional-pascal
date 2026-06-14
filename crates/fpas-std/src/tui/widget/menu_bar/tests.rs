//! Unit tests for menu bar painting and input routing.

use super::*;
use crate::console::Console;
use crate::key_event::{ConsoleKeyEvent, key_kind_index};
use crate::mouse_action_index;
use crate::{CommandId, DamageRegion, UiMouse, ViewRect};
use fpas_bytecode::SourceLocation;

use super::super::menu_popup::MenuPopupItem;

fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

fn file_item() -> MenuBarItem {
    MenuBarItem {
        label: "File".into(),
        shortcut: "F".into(),
        enabled: true,
        command_id: -1,
        submenu: vec![MenuPopupItem {
            label: "Exit".into(),
            shortcut: "X".into(),
            enabled: true,
            command_id: 1,
            separator: false,
        }],
    }
}

fn file_item_with_two_entries() -> MenuBarItem {
    MenuBarItem {
        label: "File".into(),
        shortcut: "F".into(),
        enabled: true,
        command_id: -1,
        submenu: vec![
            MenuPopupItem {
                label: "Open".into(),
                shortcut: String::new(),
                enabled: true,
                command_id: 10,
                separator: false,
            },
            MenuPopupItem {
                label: "Exit".into(),
                shortcut: "X".into(),
                enabled: true,
                command_id: 1,
                separator: false,
            },
        ],
    }
}

fn bar_rect() -> ViewRect {
    ViewRect {
        x: 0,
        y: 0,
        width: 20,
        height: 1,
    }
}

#[test]
fn menu_bar_paints_shortcut_letter_in_accel_color() {
    let mut console = Console::new();
    console.assign_crt().unwrap();
    console.begin_tui_paint(DamageRegion::FullFrame);

    MenuBarWidget::new(vec![file_item()], MenuBarStyle::default()).paint(
        &mut console,
        bar_rect(),
        DamageRegion::FullFrame,
    );

    console.finish_tui_paint(loc()).unwrap();
    assert_eq!(console.test_cell(2, 1), ('F', 4, 7));
    assert_eq!(console.test_cell(3, 1), ('i', 0, 7));
}

#[test]
fn menu_bar_mouse_move_highlights_bar_item() {
    let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
    let result = widget.handle_mouse(
        bar_rect(),
        UiMouse::new(mouse_action_index("Move"), 1, 2, 1, Default::default()),
    );
    assert_eq!(result, MenuBarMouseResult::HoverChanged);
    assert_eq!(widget.query_state().hovered_index, 0);

    let mut console = Console::new();
    console.assign_crt().unwrap();
    console.begin_tui_paint(DamageRegion::FullFrame);
    widget.paint(&mut console, bar_rect(), DamageRegion::FullFrame);
    console.finish_tui_paint(loc()).unwrap();
    assert_eq!(console.test_cell(2, 1), ('F', 7, 0));
    assert_eq!(console.test_cell(3, 1), ('i', 7, 0));
}

#[test]
fn menu_bar_alt_shortcut_opens_submenu() {
    let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
    let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
    assert_eq!(widget.handle_key(&key), MenuBarMouseResult::HoverChanged);
    assert_eq!(widget.damage_rects(bar_rect()).len(), 2);
}

#[test]
fn menu_bar_query_state_reflects_open_submenu() {
    let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
    let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
    let _ = widget.handle_key(&key);

    let state = widget.query_state();
    assert!(state.menu_active);
    assert_eq!(state.hovered_index, 0);
    assert!(state.submenu_open);
    assert_eq!(state.submenu_bar_index, 0);
    assert_eq!(state.selected_entry, 0);
}

#[test]
fn menu_bar_submenu_enter_dispatches_command() {
    let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
    let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
    let _ = widget.handle_key(&key);
    let enter = ConsoleKeyEvent::new(key_kind_index("Enter"), '\0', false, false, false, false);
    assert_eq!(
        widget.handle_key(&enter),
        MenuBarMouseResult::Command(CommandId(1))
    );
    assert_eq!(widget.damage_rects(bar_rect()).len(), 1);
}

#[test]
fn menu_bar_f10_opens_first_submenu() {
    let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
    let key = ConsoleKeyEvent::new(key_kind_index("F10"), '\0', false, false, false, false);
    assert_eq!(widget.handle_key(&key), MenuBarMouseResult::HoverChanged);
    assert_eq!(widget.damage_rects(bar_rect()).len(), 2);
}

#[test]
fn menu_bar_submenu_click_dispatches_command() {
    let mut widget = MenuBarWidget::new(vec![file_item()], MenuBarStyle::default());
    let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
    let _ = widget.handle_key(&key);

    // Popup entry row: view y=2 → console y=3 (one-based).
    let result = widget.handle_mouse(
        bar_rect(),
        UiMouse::new(mouse_action_index("Down"), 1, 2, 3, Default::default()),
    );
    assert_eq!(result, MenuBarMouseResult::Command(CommandId(1)));
    assert_eq!(widget.damage_rects(bar_rect()).len(), 1);
}

#[test]
fn menu_bar_submenu_mouse_move_changes_selection() {
    let mut widget =
        MenuBarWidget::new(vec![file_item_with_two_entries()], MenuBarStyle::default());
    let key = ConsoleKeyEvent::new(key_kind_index("Character"), 'f', false, false, true, false);
    let _ = widget.handle_key(&key);
    assert_eq!(widget.query_state().selected_entry, 0);

    let result = widget.handle_mouse(
        bar_rect(),
        UiMouse::new(mouse_action_index("Move"), 1, 2, 4, Default::default()),
    );
    assert_eq!(result, MenuBarMouseResult::HoverChanged);
    assert_eq!(widget.query_state().selected_entry, 1);
}

#[test]
fn menu_bar_click_dispatches_command() {
    let mut widget = MenuBarWidget::new(
        vec![MenuBarItem {
            label: "Quit".into(),
            shortcut: String::new(),
            enabled: true,
            command_id: 99,
            submenu: Vec::new(),
        }],
        MenuBarStyle::default(),
    );
    let result = widget.handle_mouse(
        bar_rect(),
        UiMouse::new(mouse_action_index("Down"), 1, 2, 1, Default::default()),
    );
    assert_eq!(result, MenuBarMouseResult::Command(CommandId(99)));
}
