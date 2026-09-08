"""Validate generated Functional Pascal JSONL dataset artifacts."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TRAINING = ROOT / "training" / "fpas"
DATA = TRAINING / "data"
MANIFEST = TRAINING / "manifests" / "dataset-v1.json"
SPLITS = ("train", "validation", "test")
ROLES = ["system", "user", "assistant"]
SOURCE_FILE = re.compile(r"^(?:program|unit)\s+", re.IGNORECASE)
FORBIDDEN_CODE = (
    (re.compile(r"\bStd\.WriteLn\b"), "Std.WriteLn is not an FPAS unit"),
    (re.compile(r"\.fpp\b", re.IGNORECASE), "FPAS source files use .fpas"),
    (
        re.compile(r"^\s*for\s+[A-Za-z_][A-Za-z0-9_]*\s+in\s+", re.IGNORECASE | re.MULTILINE),
        "FPAS for-in variables require a type",
    ),
)


def fail(path: Path, line_number: int, message: str) -> None:
    """Stop validation with a precise record location."""
    raise SystemExit(f"{path.name}:{line_number}: {message}")


def validate() -> dict[str, int]:
    """Validate schema, FPAS identity, common syntax traps, and split isolation."""
    if not MANIFEST.is_file():
        raise SystemExit(f"missing dataset manifest: {MANIFEST.as_posix()}")
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    counts: dict[str, int] = {}
    seen: dict[str, tuple[str, int]] = {}
    for split in SPLITS:
        path = DATA / f"{split}.jsonl"
        if not path.is_file():
            raise SystemExit(f"missing dataset split: {path.as_posix()}")
        count = 0
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                fail(path, line_number, f"invalid JSON: {error}")
            if set(record) != {"messages"}:
                fail(path, line_number, "record must contain only messages")
            messages = record.get("messages")
            if not isinstance(messages, list) or len(messages) != 3:
                fail(path, line_number, "expected exactly three messages")
            if [item.get("role") for item in messages] != ROLES:
                fail(path, line_number, "expected system/user/assistant roles")
            if any(not isinstance(item.get("content"), str) or not item["content"].strip() for item in messages):
                fail(path, line_number, "message content must be non-empty text")
            system, user, assistant = (item["content"] for item in messages)
            if "Functional Pascal" not in system or "Free Pascal" not in system or "Delphi" not in system:
                fail(path, line_number, "system message must identify FPAS and reject other Pascal dialects")
            digest = hashlib.sha256(line.encode("utf-8")).hexdigest()
            if digest in seen:
                previous_split, previous_line = seen[digest]
                fail(path, line_number, f"duplicate of {previous_split}.jsonl:{previous_line}")
            seen[digest] = (split, line_number)
            if SOURCE_FILE.search(assistant.lstrip()):
                for pattern, reason in FORBIDDEN_CODE:
                    if pattern.search(assistant):
                        fail(path, line_number, reason)
            count += 1
        if count == 0:
            raise SystemExit(f"empty dataset split: {path.name}")
        counts[split] = count

    if counts != manifest.get("split_counts"):
        raise SystemExit("manifest split counts do not match generated JSONL files")
    if manifest.get("curated_training_count", 0) < 8:
        raise SystemExit("dataset must retain the curated FPAS correction examples")
    return counts


if __name__ == "__main__":
    print(json.dumps(validate(), sort_keys=True))
