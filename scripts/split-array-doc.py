#!/usr/bin/env python3
"""Split collections/array.md into themed pages under docs/pascal/std/collections/array/."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs/pascal/std/collections/array.md"
OUT = ROOT / "docs/pascal/std/collections/array"


def slice_lines(lines: list[str], start: int, end: int) -> str:
    return "".join(lines[start - 1 : end]).strip()


def page(title: str, body: str, see: list[str]) -> str:
    links = "\n".join(f"- {line}" for line in see)
    return f"# {title}\n\n{body}\n\n## See also\n\n{links}\n"


def main() -> None:
    lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
    OUT.mkdir(parents=True, exist_ok=True)

    intro = slice_lines(lines, 3, 12)
    importing = slice_lines(lines, 15, 21)
    quick = slice_lines(lines, 23, 52)
    impl = slice_lines(lines, 317, 321)

    readme = f"""# `Std.Array`

{intro}

## Importing and names

{importing.split('## Importing and names', 1)[-1].strip()}

## Quick reference

{quick.split('## Quick reference', 1)[-1].strip()}

## Topics

| Topic | Description |
|-------|-------------|
| [Basics](basics.md) | `Length`, `Sort`, `Reverse`, search, `Slice` |
| [Mutating](mutating.md) | `Push`, `Pop` |
| [Higher-order](higher-order.md) | `Map`, `Filter`, `Reduce`, `Find`, `Any`, `All` |
| [Combine and iterate](combine.md) | `Concat`, `FlatMap`, `Fill`, `ForEach` |

## Implementation (contributors)

{impl}

## See also

- [Collections index](../README.md)
- [Standard library index](../../README.md)
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8", newline="\n")

    pages = {
        "basics.md": (
            "Basics",
            slice_lines(lines, 55, 121),
            ["[Array overview](README.md)", "[Mutating](mutating.md)", "[Collections index](../README.md)"],
        ),
        "mutating.md": (
            "Mutating",
            slice_lines(lines, 124, 146),
            ["[Array overview](README.md)", "[Higher-order](higher-order.md)", "[Collections index](../README.md)"],
        ),
        "higher-order.md": (
            "Higher-order",
            slice_lines(lines, 149, 258),
            ["[Array overview](README.md)", "[Combine and iterate](combine.md)", "[`Std.Option`](../../result/option.md)"],
        ),
        "combine.md": (
            "Combine and iterate",
            slice_lines(lines, 262, 311),
            ["[Array overview](README.md)", "[Higher-order](higher-order.md)", "[Collections index](../README.md)"],
        ),
    }

    for name, (title, body, see) in pages.items():
        (OUT / name).write_text(page(title, body, see), encoding="utf-8", newline="\n")

    SRC.unlink()
    print("Split array docs into", OUT)


if __name__ == "__main__":
    main()
