#!/usr/bin/env python3
"""Split graph/app.md into themed pages under docs/pascal/std/graph/app/."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs/pascal/std/graph/app.md"
OUT = ROOT / "docs/pascal/std/graph/app"


def slice_lines(lines: list[str], start: int, end: int) -> str:
    return "".join(lines[start - 1 : end]).strip()


def page(title: str, body: str, see: list[str]) -> str:
    links = "\n".join(f"- {line}" for line in see)
    return f"# {title}\n\n{body}\n\n## See also\n\n{links}\n"


def main() -> None:
    lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
    OUT.mkdir(parents=True, exist_ok=True)

    status = slice_lines(lines, 1, 4)
    model = slice_lines(lines, 6, 19)

    readme = f"""# `Std.Graph` — dispatch-mode application

{status}

{model}

| Topic | Description |
|-------|-------------|
| [Handlers](handlers.md) | `ApplicationHandlers` record |
| [Lifecycle](lifecycle.md) | `Open`, `Configure`, `Run`, `ExitReason` |
| [VM bridge](vm-bridge.md) | Graph intrinsics and test entrypoints |

## Implementation (contributors)

Keep types and routines in [`loaded/graph/`](../../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs) aligned with these pages. See [AGENTS.md](../../../../AGENTS.md).

## See also

- [Session API](../session.md)
- [`Std.Test`](../../testing/test.md)
- [Graphics index](../README.md)
- [Standard library index](../../README.md)
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8", newline="\n")

    pages = {
        "handlers.md": (
            "Handlers",
            slice_lines(lines, 22, 37),
            ["[Hosted dispatch overview](README.md)", "[Lifecycle](lifecycle.md)", "[Session API](../session.md)"],
        ),
        "lifecycle.md": (
            "Lifecycle",
            slice_lines(lines, 40, 76)
            + "\n\n"
            + "## Example\n\n"
            + slice_lines(lines, 97, 99).replace("## Example\n\n", ""),
            ["[Handlers](handlers.md)", "[VM bridge](vm-bridge.md)", "[Hosted dispatch overview](README.md)"],
        ),
        "vm-bridge.md": (
            "VM bridge",
            slice_lines(lines, 79, 96),
            ["[`Std.Test`](../../testing/test.md)", "[Lifecycle](lifecycle.md)", "[Hosted dispatch overview](README.md)"],
        ),
    }

    for name, (title, body, see) in pages.items():
        (OUT / name).write_text(page(title, body, see), encoding="utf-8", newline="\n")

    SRC.unlink()
    print("Split graph app docs into", OUT)


if __name__ == "__main__":
    main()
