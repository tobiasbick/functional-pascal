#!/usr/bin/env python3
"""Split text/str.md into themed pages under docs/pascal/std/text/str/."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs/pascal/std/text/str.md"
OUT = ROOT / "docs/pascal/std/text/str"


def slice_lines(lines: list[str], start: int, end: int) -> str:
    return "".join(lines[start - 1 : end]).strip()


def page(title: str, body: str, see: list[str]) -> str:
    links = "\n".join(f"- {line}" for line in see)
    return f"# {title}\n\n{body}\n\n## See also\n\n{links}\n"


def main() -> None:
    lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
    OUT.mkdir(parents=True, exist_ok=True)

    intro = slice_lines(lines, 3, 11)
    importing = slice_lines(lines, 17, 22)
    quick = slice_lines(lines, 25, 63)
    impl = slice_lines(lines, 395, 401)

    readme = f"""# `Std.Str`

String helpers: measure, search, transform, split, and join.

{intro}

## Importing and names

{importing.split('## Importing and names', 1)[-1].strip()}

## Quick reference

{quick.split('## Quick reference', 1)[-1].strip()}

## Topics

| Topic | Description |
|-------|-------------|
| [Case and trim](case-trim.md) | `Length`, `ToUpper`, `ToLower`, trim |
| [Search](search.md) | `Contains`, `IndexOf`, `Substring`, … |
| [Split and join](split-join.md) | `Split`, `Join` |
| [Edit](edit.md) | `Replace`, `Pad*`, `Insert`, `Delete`, … |
| [Format and characters](format-chars.md) | `Format`, `Ord`, `Chr`, `IsNumeric` |

## Implementation (contributors)

{impl.split('## Implementation', 1)[-1].strip()}

## See also

- [Text and parsing index](../README.md)
- [Standard library index](../../README.md)
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8", newline="\n")

    pages = {
        "case-trim.md": (
            "Case and trim",
            slice_lines(lines, 66, 105) + "\n\n" + slice_lines(lines, 333, 349),
            ["[Str overview](README.md)", "[Search](search.md)", "[Text index](../README.md)"],
        ),
        "search.md": (
            "Search",
            slice_lines(lines, 107, 157) + "\n\n" + slice_lines(lines, 353, 360),
            ["[Str overview](README.md)", "[Split and join](split-join.md)", "[Text index](../README.md)"],
        ),
        "split-join.md": (
            "Split and join",
            slice_lines(lines, 169, 196),
            ["[Str overview](README.md)", "[Edit](edit.md)", "[Text index](../README.md)"],
        ),
        "edit.md": (
            "Edit",
            slice_lines(lines, 159, 167)
            + "\n\n"
            + slice_lines(lines, 209, 329),
            ["[Str overview](README.md)", "[Format and characters](format-chars.md)", "[Text index](../README.md)"],
        ),
        "format-chars.md": (
            "Format and characters",
            slice_lines(lines, 198, 207)
            + "\n\n"
            + slice_lines(lines, 252, 301)
            + "\n\n"
            + slice_lines(lines, 364, 393),
            ["[Str overview](README.md)", "[`Std.Conv`](../conv.md)", "[Text index](../README.md)"],
        ),
    }

    for name, (title, body, see) in pages.items():
        (OUT / name).write_text(page(title, body, see), encoding="utf-8", newline="\n")

    SRC.unlink()
    print("Split str docs into", OUT)


if __name__ == "__main__":
    main()
