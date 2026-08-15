"""Validate generated Functional Pascal JSONL dataset artifacts."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "training" / "fpas" / "data"
SPLITS = ("train", "validation", "test")


def validate() -> dict[str, int]:
    counts: dict[str, int] = {}
    for split in SPLITS:
        path = DATA / f"{split}.jsonl"
        if not path.is_file():
            raise SystemExit(f"missing dataset split: {path.as_posix()}")
        count = 0
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                raise SystemExit(f"{path.name}:{line_number}: invalid JSON: {error}") from error
            messages = record.get("messages")
            if not isinstance(messages, list) or [item.get("role") for item in messages] != ["system", "user", "assistant"]:
                raise SystemExit(f"{path.name}:{line_number}: expected system/user/assistant messages")
            if any(not isinstance(item.get("content"), str) or not item["content"].strip() for item in messages):
                raise SystemExit(f"{path.name}:{line_number}: message content must be non-empty text")
            count += 1
        if count == 0:
            raise SystemExit(f"empty dataset split: {path.name}")
        counts[split] = count
    return counts


if __name__ == "__main__":
    print(json.dumps(validate(), sort_keys=True))
