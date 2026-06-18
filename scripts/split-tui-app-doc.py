#!/usr/bin/env python3
"""Split tui/app.md into themed pages under docs/pascal/std/tui/app/."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs/pascal/std/tui/app.md"
OUT = ROOT / "docs/pascal/std/tui/app"


def slice_lines(lines: list[str], start: int, end: int) -> str:
    return "".join(lines[start - 1 : end]).strip()


def page(title: str, body: str, see: list[str]) -> str:
    links = "\n".join(f"- {line}" for line in see)
    return f"# {title}\n\n{body}\n\n## See also\n\n{links}\n"


def main() -> None:
    lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
    OUT.mkdir(parents=True, exist_ok=True)

    status = slice_lines(lines, 1, 3)

    readme = f"""# `Std.Tui` — dispatch-mode application

{status}

| Topic | Description |
|-------|-------------|
| [VM bridge](vm-bridge.md) | Intrinsics, modals, views, host widgets |
| [Lifecycle](lifecycle.md) | `Open`, `Configure`, `Run`, `Close` |
| [Handlers](handlers.md) | `On*` callbacks and registration |
| [Types](types.md) | `ApplicationHandlers`, `ExitReason`, signatures |
| [Native testing](testing.md) | `OpenForTest`, `TestPump`, `Query*` |

## Implementation (contributors)

Keep types and routines in [`loaded/tui/`](../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) aligned with these pages. See [AGENTS.md](../../../../AGENTS.md).

## See also

- [Session API](../session.md)
- [`Std.Console`](../../console/README.md)
- [Terminal UI index](../README.md)
- [TUI framework](../../../future/tui-application-framework.md)
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8", newline="\n")

    pages = {
        "vm-bridge.md": (
            slice_lines(lines, 9, 222),
            ["[Hosted dispatch overview](README.md)", "[Handlers](handlers.md)", "[Native testing](testing.md#viewid-type-decided)"],
        ),
        "lifecycle.md": (
            slice_lines(lines, 224, 230)
            + "\n\n"
            + slice_lines(lines, 232, 251),
            ["[Hosted dispatch overview](README.md)", "[Handlers](handlers.md)", "[Session API](../session.md)"],
        ),
        "handlers.md": (
            slice_lines(lines, 255, 303)
            + "\n\n"
            + slice_lines(lines, 362, 450),
            ["[Types](types.md)", "[Lifecycle](lifecycle.md)", "[Hosted dispatch overview](README.md)"],
        ),
        "types.md": (
            slice_lines(lines, 305, 360),
            ["[Handlers](handlers.md)", "[Hosted dispatch overview](README.md)"],
        ),
        "testing.md": (
            slice_lines(lines, 454, 703),
            ["[`Std.Test`](../../testing/test.md)", "[Terminal checklist](../terminal-checklist.md)", "[Hosted dispatch overview](README.md)"],
        ),
    }

    titles = {
        "vm-bridge.md": "VM bridge",
        "lifecycle.md": "Lifecycle",
        "handlers.md": "Handlers",
        "types.md": "Types and registration",
        "testing.md": "Native testing",
    }

    for name, (body, see) in pages.items():
        (OUT / name).write_text(page(titles[name], body, see), encoding="utf-8", newline="\n")

    SRC.unlink()
    print("Split tui app docs into", OUT)


if __name__ == "__main__":
    main()
