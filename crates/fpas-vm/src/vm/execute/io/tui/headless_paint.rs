//! Headless Turbo Vision desktop paint into the CRT screen buffer.
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::shared::TurboVisionObject;
use fpas_bytecode::SourceLocation;
use fpas_std::Console;

const TITLE_FG: u8 = 15;
const TITLE_BG: u8 = 1;
const TEXT_FG: u8 = 7;
const TEXT_BG: u8 = 0;

enum HeadlessPaintOp {
    Text {
        x: i16,
        y: i16,
        text: String,
        fg: u8,
        bg: u8,
    },
}

impl Worker {
    /// Repaint every on-desktop Turbo Vision window into the logical CRT buffer.
    pub(in crate::vm::execute::io::tui) fn turbo_vision_paint_headless_desktop(
        &mut self,
        _line: SourceLocation,
    ) {
        let ops = self.with_tui(|tui| {
            let mut ops = Vec::new();
            if let Some(handle) = tui.turbo_vision.menu_bar
                && let Some(TurboVisionObject::MenuBar(menu_bar)) =
                    tui.turbo_vision.objects.get(&handle)
            {
                collect_text_sequence_ops(
                    &mut ops,
                    menu_bar.bounds.x,
                    menu_bar.bounds.y,
                    menu_bar.menus.iter().map(|menu| menu.title.as_str()),
                    TITLE_FG,
                    TITLE_BG,
                );
            }
            if let Some(handle) = tui.turbo_vision.status_line
                && let Some(TurboVisionObject::StatusLine(status_line)) =
                    tui.turbo_vision.objects.get(&handle)
            {
                collect_text_sequence_ops(
                    &mut ops,
                    status_line.bounds.x,
                    status_line.bounds.y,
                    status_line.items.iter().map(|item| item.text.as_str()),
                    TEXT_FG,
                    TEXT_BG,
                );
            }
            for object in tui.turbo_vision.objects.values() {
                // Windows are shown once placed on the desktop; dialogs are shown
                // as soon as they exist (mirroring `build_turbo_vision_application`).
                let container = match object {
                    TurboVisionObject::Window(window) if window.on_desktop => {
                        Some((window.bounds, window.title.as_str(), &window.children))
                    }
                    TurboVisionObject::Dialog(dialog) => {
                        Some((dialog.bounds, dialog.title.as_str(), &dialog.children))
                    }
                    _ => None,
                };
                let Some((bounds, title, children)) = container else {
                    continue;
                };
                ops.push(HeadlessPaintOp::Text {
                    x: bounds.x.saturating_add(1),
                    y: bounds.y,
                    text: title.to_string(),
                    fg: TITLE_FG,
                    bg: TITLE_BG,
                });
                for handle in children {
                    let Some(child) = tui.turbo_vision.objects.get(handle) else {
                        continue;
                    };
                    collect_child_paint_ops(&mut ops, bounds.x, bounds.y, child);
                }
            }
            ops
        });

        self.with_console(|console| {
            console.clear_headless_screen();
            for op in ops {
                match op {
                    HeadlessPaintOp::Text { x, y, text, fg, bg } => {
                        paint_text(console, x, y, &text, fg, bg);
                    }
                }
            }
        });
    }
}

fn collect_text_sequence_ops<'a>(
    ops: &mut Vec<HeadlessPaintOp>,
    x: i16,
    y: i16,
    texts: impl IntoIterator<Item = &'a str>,
    fg: u8,
    bg: u8,
) {
    let mut current_x = x;
    for text in texts {
        if text.is_empty() {
            continue;
        }
        ops.push(HeadlessPaintOp::Text {
            x: current_x,
            y,
            text: text.to_string(),
            fg,
            bg,
        });
        current_x = current_x
            .saturating_add(i16::try_from(text.chars().count()).unwrap_or(i16::MAX))
            .saturating_add(1);
    }
}

fn collect_child_paint_ops(
    ops: &mut Vec<HeadlessPaintOp>,
    parent_x: i16,
    parent_y: i16,
    child: &TurboVisionObject,
) {
    let (local_x, local_y, text) = match child {
        TurboVisionObject::StaticText(static_text) => (
            static_text.bounds.x,
            static_text.bounds.y,
            static_text.text.clone(),
        ),
        TurboVisionObject::Button(button) => {
            (button.bounds.x, button.bounds.y, button.text.clone())
        }
        TurboVisionObject::Memo(memo) => (memo.bounds.x, memo.bounds.y, memo.text.clone()),
        TurboVisionObject::CheckBox(check_box) => {
            let marker = if check_box.checked_cell.read() {
                'X'
            } else {
                ' '
            };
            (
                check_box.bounds.x,
                check_box.bounds.y,
                format!("{marker} {}", check_box.text),
            )
        }
        TurboVisionObject::RadioButton(radio_button) => {
            let marker = if radio_button.selected_cell.read() {
                '*'
            } else {
                ' '
            };
            (
                radio_button.bounds.x,
                radio_button.bounds.y,
                format!("{marker} {}", radio_button.text),
            )
        }
        TurboVisionObject::ListBox(list_box) => {
            let Some(first) = list_box.items.first() else {
                return;
            };
            (list_box.bounds.x, list_box.bounds.y, first.clone())
        }
        _ => return,
    };
    ops.push(HeadlessPaintOp::Text {
        x: parent_x.saturating_add(local_x),
        y: parent_y.saturating_add(local_y),
        text,
        fg: TEXT_FG,
        bg: TEXT_BG,
    });
}

fn paint_text(console: &mut Console, x: i16, y: i16, text: &str, fg: u8, bg: u8) {
    let screen_width = console.screen_width();
    let screen_height = console.screen_height();
    let mut col = i64::from(x);
    let row = i64::from(y);
    if row < 1 || row > screen_height {
        return;
    }
    for ch in text.chars() {
        if col >= 1 && col <= screen_width {
            console.paint_headless_cell(col as u16, row as u16, ch, fg, bg);
        }
        col = col.saturating_add(1);
    }
}
