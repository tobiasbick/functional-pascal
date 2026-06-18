#!/usr/bin/env python3
"""Split console-full.md into themed pages under docs/pascal/std/console/."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs/pascal/std/console/console-full.md"
OUT = ROOT / "docs/pascal/std/console"


def lines_slice(all_lines: list[str], start: int, end: int) -> str:
    return "".join(all_lines[start - 1 : end])


def main() -> None:
    lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)

    header = lines_slice(lines, 1, 17)
    importing = lines_slice(lines, 19, 32)
    quick_ref = lines_slice(lines, 34, 107)
    color_note = lines_slice(lines, 105, 107)

    pages: dict[str, tuple[str, int, int]] = {
        "types.md": ("Types", 109, 279),
        "output.md": ("Output", 281, 309),
        "screen.md": ("Screen control", 311, 383),
        "colors.md": ("Colors and attributes", 384, 491),
        "screen-misc.md": ("Screen utilities", 492, 547),
        "input.md": ("Line input", 551, 579),
        "keyboard.md": ("Keyboard", 581, 640),
        "events.md": ("Terminal events", 641, 714),
        "using-together.md": ("Using text and keyboard together", 717, 723),
    }

    for name, (title, start, end) in pages.items():
        body = lines_slice(lines, start, end).strip()
        if name == "colors.md":
            body = color_note.strip() + "\n\n---\n\n" + body
        content = f"# {title}\n\n{body}\n\n## See also\n\n- [Console overview](README.md)\n"
        if name == "types.md":
            content += "- [Keyboard](keyboard.md)\n- [Terminal events](events.md)\n"
        elif name == "output.md":
            content += "- [Screen control](screen.md)\n- [Colors and attributes](colors.md)\n"
        elif name in ("keyboard.md", "events.md"):
            content += "- [Types](types.md)\n- [Using together](using-together.md)\n"
        else:
            content += "- [Quick reference](README.md#quick-reference)\n"
        (OUT / name).write_text(content, encoding="utf-8", newline="\n")

    readme = f"""# `Std.Console`

Text output, line-buffered stdin, and terminal input (CRT-style screen control plus structured key and event APIs).

{header.strip()}

---

{importing.strip()}

---

## Quick reference

Everything below requires `uses Std.Console;`.

{quick_ref.split('## Quick reference', 1)[-1].strip()}

## Topics

| Topic | Description |
|-------|-------------|
| [Output](output.md) | `Write`, `WriteLn` |
| [Screen control](screen.md) | Windows, cursor, scrolling |
| [Screen utilities](screen-misc.md) | `Delay`, cursor visibility, `TextMode`, bell |
| [Colors and attributes](colors.md) | CRT palette, RGB, 256-color |
| [Types](types.md) | `KeyEvent`, `Event`, enums |
| [Line input](input.md) | `ReadLn`, `Read` |
| [Keyboard](keyboard.md) | `ReadKey`, `ReadKeyEvent`, `KeyPressed` |
| [Terminal events](events.md) | `ReadEvent`, raw mode, alt screen |
| [Using together](using-together.md) | Mixing line and key input |

## See also

- [`Std.Tui`](../tui/session.md) — hosted terminal applications
- [Standard library index](../README.md)

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| User-facing registration | [`loaded/console.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/console.rs) |
| Console backend | [`console/mod.rs`](../../../../crates/fpas-std/src/console/mod.rs) |
| Key and event types | [`key_event.rs`](../../../../crates/fpas-std/src/key_event.rs), [`console_event.rs`](../../../../crates/fpas-std/src/console_event.rs) |
| Bytecode / VM | [`intrinsic/mod.rs`](../../../../crates/fpas-bytecode/src/intrinsic/mod.rs), [`vm/mod.rs`](../../../../crates/fpas-vm/src/vm/mod.rs) |
| Code generation | [`std_calls/console/`](../../../../crates/fpas-compiler/src/compiler/std_calls/console/mod.rs) |
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8", newline="\n")
    SRC.unlink()
    print("Split console docs into", OUT)


if __name__ == "__main__":
    main()
