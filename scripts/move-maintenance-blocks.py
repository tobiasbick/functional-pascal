#!/usr/bin/env python3
"""Move top Maintenance blocks to ## Implementation (contributors) at page bottom."""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STD = ROOT / "docs/pascal/std"

TOP_BLOCK = re.compile(
    r"\n\*\*Maintenance \(implementers only\):\*\*([^\n]+)\n\n---\n\n",
    re.MULTILINE,
)


def main() -> None:
    n = 0
    for path in STD.rglob("*.md"):
        if path.name == "README.md" and path.parent == STD:
            continue
        text = path.read_text(encoding="utf-8")
        match = TOP_BLOCK.search(text)
        if not match:
            continue
        note = match.group(1).strip()
        text = TOP_BLOCK.sub("\n\n", text, count=1)

        impl_body = f"Keep implementation aligned with source paths referenced in the original maintenance note: {note}"
        if "## Implementation (contributors)" in text:
            pass  # top block removed; table already present
        elif "## See also" in text:
            text = text.replace(
                "## See also",
                f"## Implementation (contributors)\n\n{impl_body}\n\n## See also",
                1,
            )
        else:
            text = text.rstrip() + f"\n\n## Implementation (contributors)\n\n{impl_body}\n"

        path.write_text(text, encoding="utf-8", newline="\n")
        n += 1
    print(f"Moved maintenance blocks in {n} files")


if __name__ == "__main__":
    main()
