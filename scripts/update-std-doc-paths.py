#!/usr/bin/env python3
"""Update docs/pascal/std/* path references after directory restructure."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Longest paths first.
REPLACEMENTS: list[tuple[str, str]] = [
    ("docs/pascal/std/tui-terminal-checklist.md", "docs/pascal/std/tui/terminal-checklist.md"),
    ("docs/pascal/std/tui-app.md", "docs/pascal/std/tui/app/README.md"),
    ("docs/pascal/std/tui/app.md", "docs/pascal/std/tui/app/README.md"),
    ("docs/pascal/std/graph-app.md", "docs/pascal/std/graph/app.md"),
    ("docs/pascal/std/console.md", "docs/pascal/std/console/README.md"),
    ("docs/pascal/std/tui.md", "docs/pascal/std/tui/session.md"),
    ("docs/pascal/std/graph.md", "docs/pascal/std/graph/session.md"),
    ("docs/pascal/std/args.md", "docs/pascal/std/host/args.md"),
    ("docs/pascal/std/env.md", "docs/pascal/std/host/env.md"),
    ("docs/pascal/std/fs.md", "docs/pascal/std/host/fs.md"),
    ("docs/pascal/std/path.md", "docs/pascal/std/host/path.md"),
    ("docs/pascal/std/proc.md", "docs/pascal/std/host/proc.md"),
    ("docs/pascal/std/time.md", "docs/pascal/std/host/time.md"),
    ("docs/pascal/std/str.md", "docs/pascal/std/text/str/README.md"),
    ("docs/pascal/std/text/str.md", "docs/pascal/std/text/str/README.md"),
    ("docs/pascal/std/conv.md", "docs/pascal/std/text/conv.md"),
    ("docs/pascal/std/parse.md", "docs/pascal/std/text/parse.md"),
    ("docs/pascal/std/json.md", "docs/pascal/std/text/json.md"),
    ("docs/pascal/std/array.md", "docs/pascal/std/collections/array.md"),
    ("docs/pascal/std/dict.md", "docs/pascal/std/collections/dict.md"),
    ("docs/pascal/std/math.md", "docs/pascal/std/numeric/math.md"),
    ("docs/pascal/std/random.md", "docs/pascal/std/numeric/random.md"),
    ("docs/pascal/std/result.md", "docs/pascal/std/result/result.md"),
    ("docs/pascal/std/option.md", "docs/pascal/std/result/option.md"),
    ("docs/pascal/std/task.md", "docs/pascal/std/concurrency/task.md"),
    ("docs/pascal/std/test.md", "docs/pascal/std/testing/test.md"),
    # Relative markdown (docs tree)
    ("../std/tui-terminal-checklist.md", "../std/tui/terminal-checklist.md"),
    ("../std/tui-app.md", "../std/tui/app/README.md"),
    ("../std/tui/app.md", "../std/tui/app/README.md"),
    ("../std/graph-app.md", "../std/graph/app.md"),
    ("../std/console.md", "../std/console/README.md"),
    ("../std/tui.md", "../std/tui/session.md"),
    ("../std/graph.md", "../std/graph/session.md"),
    ("../std/args.md", "../std/host/args.md"),
    ("../std/env.md", "../std/host/env.md"),
    ("../std/fs.md", "../std/host/fs.md"),
    ("../std/path.md", "../std/host/path.md"),
    ("../std/proc.md", "../std/host/proc.md"),
    ("../std/time.md", "../std/host/time.md"),
    ("../std/str.md", "../std/text/str/README.md"),
    ("../std/text/str.md", "../std/text/str/README.md"),
    ("../std/conv.md", "../std/text/conv.md"),
    ("../std/parse.md", "../std/text/parse.md"),
    ("../std/json.md", "../std/text/json.md"),
    ("../std/array.md", "../std/collections/array.md"),
    ("../std/dict.md", "../std/collections/dict.md"),
    ("../std/math.md", "../std/numeric/math.md"),
    ("../std/random.md", "../std/numeric/random.md"),
    ("../std/result.md", "../std/result/result.md"),
    ("../std/option.md", "../std/result/option.md"),
    ("../std/task.md", "../std/concurrency/task.md"),
    ("../std/test.md", "../std/testing/test.md"),
    ("std/tui-app.md", "std/tui/app/README.md"),
    ("std/tui/app.md", "std/tui/app/README.md"),
    ("std/console.md", "std/console/README.md"),
    ("std/tui.md", "std/tui/session.md"),
    ("std/graph.md", "std/graph/session.md"),
    ("(console.md)", "(console/README.md)"),
    ("(tui-app.md)", "(tui/app/README.md)"),
    ("(tui/app.md)", "(tui/app/README.md)"),
    ("(graph-app.md)", "(graph/app.md)"),
    ("(tui.md)", "(tui/session.md)"),
    ("(graph.md)", "(graph/session.md)"),
]

SKIP_DIRS = {".git", "target", "node_modules"}


def patch_file(path: Path) -> bool:
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return False
    original = text
    for old, new in REPLACEMENTS:
        text = text.replace(old, new)
    if text != original:
        path.write_text(text, encoding="utf-8", newline="\n")
        return True
    return False


def fix_crate_depth(path: Path) -> bool:
    """Add one ../ segment for files moved one level deeper under std/."""
    if "docs/pascal/std/" not in path.as_posix().replace("\\", "/"):
        return False
    rel = path.relative_to(ROOT / "docs/pascal/std")
    if rel.parts[0] == "README.md" or len(rel.parts) == 1:
        return False
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return False
    # Only bump paths that still use three-level escape from unit pages.
    patched = text.replace("](../../../crates/", "](../../../../crates/")
    patched = patched.replace("](../../../examples/", "](../../../../examples/")
    patched = patched.replace("](../../../AGENTS.md", "](../../../../AGENTS.md")
    patched = patched.replace("](../../future/", "](../../../future/")
    patched = patched.replace("](../language/", "](../../language/")
    patched = patched.replace("](../program-structure/", "](../../program-structure/")
    if patched != text:
        path.write_text(patched, encoding="utf-8", newline="\n")
        return True
    return False


def main() -> None:
    changed = 0
    for path in ROOT.rglob("*"):
        if path.is_dir() or any(part in SKIP_DIRS for part in path.parts):
            continue
        if path.suffix not in {".md", ".rs", ".mdc", ".fpas", ".ebnf", ".toml"}:
            continue
        if patch_file(path):
            changed += 1
    for path in (ROOT / "docs/pascal/std").rglob("*.md"):
        if fix_crate_depth(path):
            changed += 1
    print(f"Updated {changed} files")


if __name__ == "__main__":
    main()
