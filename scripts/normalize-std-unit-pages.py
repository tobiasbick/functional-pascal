#!/usr/bin/env python3
"""Normalize std unit page footers to match language doc style."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STD = ROOT / "docs/pascal/std"

AREA_INDEX: dict[str, str] = {
    "host": "Host I/O",
    "text": "Text and parsing",
    "collections": "Collections",
    "numeric": "Numeric",
    "result": "Result and Option",
    "concurrency": "Concurrency",
    "testing": "Testing",
    "tui": "Terminal UI",
    "graph": "Graphics",
}

SKIP = {"README.md", "terminal-checklist.md"}


def area_for(path: Path) -> tuple[str, str] | None:
    rel = path.relative_to(STD)
    if len(rel.parts) < 2:
        return None
    area = rel.parts[0]
    if area not in AREA_INDEX:
        return None
    return area, AREA_INDEX[area]


def normalize(path: Path) -> bool:
    if path.name in SKIP or path.name == "README.md" and path.parent == STD:
        return False
    text = path.read_text(encoding="utf-8")
    original = text

    text = re.sub(r"\n\[← Standard library index\]\(README\.md\)\s*$", "", text)
    text = re.sub(
        r"\n## Implementation map \(.*?\)\n",
        "\n## Implementation (contributors)\n",
        text,
    )

    if "## See also" not in text:
        area = area_for(path)
        see = ["\n## See also\n"]
        if area:
            slug, title = area
            see.append(f"- [{title} index](README.md)")
            see.append("- [Standard library index](../README.md)")
        elif path.parent.name == "console":
            see.append("- [Console overview](README.md)")
            see.append("- [Standard library index](../README.md)")
        text = text.rstrip() + "\n" + "\n".join(see) + "\n"

    if text != original:
        path.write_text(text, encoding="utf-8", newline="\n")
        return True
    return False


def main() -> None:
    n = 0
    for path in STD.rglob("*.md"):
        if normalize(path):
            n += 1
    print(f"Normalized {n} files")


if __name__ == "__main__":
    main()
