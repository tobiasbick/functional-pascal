#!/usr/bin/env python3
"""Normalize std documentation style: footers, Implementation tables, links."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STD = ROOT / "docs/pascal/std"

PROSE_IMPL = re.compile(
    r"## Implementation \(contributors\)\n\n"
    r"Keep implementation aligned with source paths referenced in the original maintenance note: [^\n]+\n",
    re.MULTILINE,
)

HOSTED_PROSE = re.compile(
    r"## Implementation \(contributors\)\n\n"
    r"Keep types and routines in \[[^\]]+\]\([^\)]+\) aligned with these pages\. See \[[^\]]+\]\([^\)]+\)\.\n",
    re.MULTILINE,
)

STALE_STD_FOOTER = re.compile(
    r"\n\[[←<-] Standard library index\]\(README\.md\)\s*\n",
    re.MULTILINE,
)

# file (relative to STD) -> Implementation table body (without heading)
IMPL_TABLES: dict[str, str] = {
    "collections/dict.md": """| Concern | Location |
|---------|----------|
| Registration | [`std_registry/builtins/dict.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/dict.rs) |
| Runtime | [`dict.rs`](../../../../crates/fpas-std/src/dict.rs) |
| Compiler | [`std_calls/dict.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/dict.rs) |
| Intrinsics | [`intrinsic/mod.rs`](../../../../crates/fpas-bytecode/src/intrinsic/mod.rs) |""",
    "text/parse.md": """| Concern | Location |
|---------|----------|
| Registration | [`std_registry/loaded/parse.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/parse.rs) |
| Runtime | [`parse.rs`](../../../../crates/fpas-std/src/parse.rs) |
| Compiler | [`std_calls/parse.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/parse.rs) |
| Shared text | [`intrinsics.rs`](../../../../crates/fpas-std/src/intrinsics.rs) |
| Intrinsics | [`intrinsic/parse.rs`](../../../../crates/fpas-bytecode/src/intrinsic/parse.rs) |""",
    "text/json.md": """| Concern | Location |
|---------|----------|
| Registration | [`loaded/json.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/json.rs) |
| Runtime | [`json.rs`](../../../../crates/fpas-std/src/json.rs) |
| Compiler | [`std_calls/json.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/json.rs) |
| Intrinsics | [`intrinsic/json.rs`](../../../../crates/fpas-bytecode/src/intrinsic/json.rs) |""",
    "testing/test.md": """| Concern | Location |
|---------|----------|
| Registration | [`std_registry/loaded/test.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/test.rs) |
| Compiler | [`std_calls/test.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/test.rs) |
| Runtime | [`test/`](../../../../crates/fpas-std/src/test/) |
| Intrinsics | [`intrinsic/test.rs`](../../../../crates/fpas-bytecode/src/intrinsic/test.rs) |""",
    "concurrency/task.md": """| Concern | Location |
|---------|----------|
| Registration | [`loaded/channel_task.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/channel_task.rs), [`builtins/channel_task.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/channel_task.rs) |
| Compiler | [`std_calls/task.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/task.rs), [`compiler/stmt/concurrency/mod.rs`](../../../../crates/fpas-compiler/src/compiler/stmt/concurrency/mod.rs), [`compiler/expr/mod.rs`](../../../../crates/fpas-compiler/src/compiler/expr/mod.rs) |
| Bytecode | [`chunk.rs`](../../../../crates/fpas-bytecode/src/chunk.rs), [`intrinsic/mod.rs`](../../../../crates/fpas-bytecode/src/intrinsic/mod.rs) |
| VM | [`shared.rs`](../../../../crates/fpas-vm/src/vm/shared.rs), [`tasks/spawn.rs`](../../../../crates/fpas-vm/src/vm/execute/concurrency/tasks/spawn.rs), [`tasks/wait.rs`](../../../../crates/fpas-vm/src/vm/execute/concurrency/tasks/wait.rs), [`tasks/scheduling.rs`](../../../../crates/fpas-vm/src/vm/execute/concurrency/tasks/scheduling.rs), [`concurrency/mod.rs`](../../../../crates/fpas-vm/src/vm/execute/concurrency/mod.rs) |""",
    "tui/session.md": """| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/tui/mod.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) |
| Std units | [`std_units/mod.rs`](../../../../crates/fpas-std/src/std_units/mod.rs) |""",
    "tui/app/README.md": """| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/tui/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/loaded/tui/mod.rs) |
| Contributor guide | [AGENTS.md](../../../../../AGENTS.md) |""",
    "graph/app/README.md": """| Concern | Location |
|---------|----------|
| Sema registry | [`loaded/graph/mod.rs`](../../../../../crates/fpas-sema/src/std_registry/loaded/graph/mod.rs) |
| Contributor guide | [AGENTS.md](../../../../../AGENTS.md) |""",
}


def replace_impl_table(text: str, rel: str) -> str:
    if rel not in IMPL_TABLES:
        return text
    table = IMPL_TABLES[rel]
    text = PROSE_IMPL.sub(f"## Implementation (contributors)\n\n{table}\n", text)
    text = HOSTED_PROSE.sub(f"## Implementation (contributors)\n\n{table}\n", text)
    text = re.sub(
        r"(## Implementation \(contributors\)\n\n)(Keep [^\n]+\n)",
        rf"\1{table}\n",
        text,
        count=1,
    )
    return text


def ensure_std_index_in_see_also(text: str, std_index_line: str) -> str:
    if "## See also" not in text or std_index_line in text:
        return text
    return text.rstrip() + f"\n{std_index_line}\n"


def fix_file(path: Path) -> bool:
    rel = path.relative_to(STD).as_posix()
    text = path.read_text(encoding="utf-8")
    original = text

    text = text.replace("## Implementation map (contributors)", "## Implementation (contributors)")
    text = STALE_STD_FOOTER.sub("\n", text)
    text = replace_impl_table(text, rel)

    # Global link fixes
    text = text.replace("(tui-terminal-checklist.md)", "(../terminal-checklist.md)")
    text = text.replace("[tui-terminal-checklist.md]", "[Terminal checklist](../terminal-checklist.md)")

    if rel.startswith("graph/app/") or rel.startswith("tui/app/"):
        text = text.replace("../../../testing/test.md", "../../testing/test.md")

    if rel == "graph/app/README.md":
        text = re.sub(r"(# `Std\.Graph` — dispatch-mode application\n\n)+", r"# `Std.Graph` — dispatch-mode application\n\n", text)

    if rel == "tui/app/README.md":
        text = text.replace("../../../../../../crates/", "../../../../../crates/")
        text = text.replace("../../../../../../AGENTS.md", "../../../../../AGENTS.md")
        text = text.replace("../../../../../future/", "../../../../future/")
        text = re.sub(
            r"`\[docs/future/tui-application-framework\.md\]\(([^)]+)\)`",
            r"[TUI framework](../../../../future/tui-application-framework.md)",
            text,
        )
        text = text.replace(
            "[TUI framework](../../../../../future/tui-application-framework.md)",
            "[TUI framework](../../../../future/tui-application-framework.md)",
        )

    if rel == "text/str/README.md":
        text = text.replace(
            "| printf-style string formatting |\n**Indexing:**",
            "| printf-style string formatting |\n\n**Indexing:**",
        )

    if rel.startswith("collections/array/") and path.name != "README.md":
        text = text.replace(
            "- [Collections index](../README.md)",
            "- [Collections index](../../README.md)",
        )

    if rel == "testing/README.md":
        text = text.replace("| Topic | Description |", "| Unit | Description |")

    if rel == "console/README.md":
        # Move Implementation block before See also
        impl_match = re.search(
            r"\n## Implementation \(contributors\)\n(?:.*?\n)(?=\Z)",
            text,
            re.DOTALL,
        )
        see_match = re.search(r"\n## See also\n.*?(?=\n## Implementation|\Z)", text, re.DOTALL)
        if impl_match and see_match and see_match.start() < impl_match.start():
            impl_block = impl_match.group(0)
            see_block = see_match.group(0)
            body = text[: see_match.start()] + impl_block + see_block
            text = body.rstrip() + "\n"

    if rel == "tui/session.md":
        text = ensure_std_index_in_see_also(text, "- [Standard library index](../README.md)")

    if rel == "tui/app/README.md":
        text = ensure_std_index_in_see_also(text, "- [Standard library index](../../README.md)")

    if rel == "tui/terminal-checklist.md" and "## See also" not in text:
        text = text.rstrip() + """

## See also

- [Native testing](app/testing.md)
- [Hosted dispatch](app/README.md)
- [Terminal UI index](README.md)
- [Standard library index](../README.md)
"""

    if text != original:
        path.write_text(text, encoding="utf-8", newline="\n")
        return True
    return False


def main() -> None:
    n = 0
    for path in STD.rglob("*.md"):
        if fix_file(path):
            n += 1
            print("fixed", path.relative_to(ROOT))
    print(f"Normalized {n} files")


if __name__ == "__main__":
    main()
