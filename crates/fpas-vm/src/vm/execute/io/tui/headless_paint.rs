//! Headless Turbo Vision desktop paint into the CRT screen buffer.
//!
//! **Documentation:** `docs/future/turbo-vision-4-rust/07-post-migration-improvements.md` (Phase C)

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
            for object in tui.turbo_vision.objects.values() {
                let TurboVisionObject::Window(window) = object else {
                    continue;
                };
                if !window.on_desktop {
                    continue;
                }
                ops.push(HeadlessPaintOp::Text {
                    x: window.bounds.x.saturating_add(1),
                    y: window.bounds.y,
                    text: window.title.clone(),
                    fg: TITLE_FG,
                    bg: TITLE_BG,
                });
                for handle in &window.children {
                    let Some(child) = tui.turbo_vision.objects.get(handle) else {
                        continue;
                    };
                    collect_child_paint_ops(&mut ops, window.bounds.x, window.bounds.y, child);
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
            static_text.text.as_str(),
        ),
        TurboVisionObject::Button(button) => {
            (button.bounds.x, button.bounds.y, button.text.as_str())
        }
        TurboVisionObject::Memo(memo) => (memo.bounds.x, memo.bounds.y, memo.text.as_str()),
        TurboVisionObject::CheckBox(check_box) => (
            check_box.bounds.x,
            check_box.bounds.y,
            check_box.text.as_str(),
        ),
        TurboVisionObject::RadioButton(radio_button) => (
            radio_button.bounds.x,
            radio_button.bounds.y,
            radio_button.text.as_str(),
        ),
        _ => return,
    };
    ops.push(HeadlessPaintOp::Text {
        x: parent_x.saturating_add(local_x),
        y: parent_y.saturating_add(local_y),
        text: text.to_string(),
        fg: TEXT_FG,
        bg: TEXT_BG,
    });
}

fn paint_text(console: &mut Console, x: i16, y: i16, text: &str, fg: u8, bg: u8) {
    let mut col = i64::from(x);
    let row = i64::from(y);
    if row < 1 {
        return;
    }
    for ch in text.chars() {
        if col >= 1 {
            console.paint_headless_cell(col as u16, row as u16, ch, fg, bg);
        }
        col = col.saturating_add(1);
    }
}
